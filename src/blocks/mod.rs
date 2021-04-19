//! Handling, creating and parsing blocks.
mod block_hash;
mod change_block;
mod open_block;
mod receive_block;
mod send_block;
mod state_block;

#[cfg(feature = "node")]
use crate::node::Wire;

#[cfg(feature = "node")]
use crate::node::Header;

use crate::encoding::blake2b;
use crate::keys::public::to_address;
use crate::network::Network;
use crate::{Private, Public, Rai, Signature, Work};
use anyhow::{anyhow, Context};
pub use block_hash::BlockHash;
pub use change_block::ChangeBlock;
pub use open_block::OpenBlock;
pub use receive_block::ReceiveBlock;
pub use send_block::SendBlock;
use serde;
use serde::{Deserialize, Serialize};
pub(crate) use state_block::deserialize_to_unsure_link;
pub use state_block::{Link, StateBlock, Subtype};
use std::convert::TryFrom;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BlockType {
    Invalid,
    NotABlock,
    Send,
    Receive,
    Open,
    Change,
    State,
}

impl BlockType {
    pub fn as_u8(&self) -> u8 {
        match self {
            BlockType::Invalid => 0,
            BlockType::NotABlock => 1,
            BlockType::Send => 2,
            BlockType::Receive => 3,
            BlockType::Open => 4,
            BlockType::Change => 5,
            BlockType::State => 6,
        }
    }
}

impl TryFrom<u8> for BlockType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use BlockType::*;
        Ok(match value {
            0 => Invalid,
            1 => NotABlock,
            2 => Send,
            3 => Receive,
            4 => Open,
            5 => Change,
            6 => State,
            _ => return Err(anyhow!("Invalid block type: {}", value)),
        })
    }
}

/// For "holding" deserialized blocks that we can't convert to `Block` yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockHolder {
    Send(SendBlock),
    Receive(ReceiveBlock),
    Open(OpenBlock),
    Change(ChangeBlock),
    State(StateBlock),
}

#[cfg(feature = "node")]
impl Wire for BlockHolder {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        debug_assert!(header.is_some());
        let context = "Deserialize BlockHolder";

        let holder = match header
            .as_ref()
            .unwrap()
            .ext()
            .block_type()
            .context(context)?
        {
            BlockType::State => {
                BlockHolder::State(Wire::deserialize(header, data).context(context)?)
            }
            BlockType::Send => BlockHolder::Send(Wire::deserialize(header, data).context(context)?),
            _ => todo!(),
        };
        Ok(holder)
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        debug_assert!(header.is_some());
        match header.as_ref().unwrap().ext().block_type()? {
            BlockType::State => StateBlock::len(header),
            BlockType::Send => SendBlock::len(header),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Previous {
    Block(BlockHash),
    Open,
}

impl Previous {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Previous::Block(b) => b.as_bytes().to_vec(),
            Previous::Open => BlockHash::zero().as_bytes().to_vec(),
        }
    }
}

impl TryFrom<&[u8]> for Previous {
    type Error = crate::Error;

    fn try_from(slice_of_bytes: &[u8]) -> crate::Result<Self> {
        if slice_of_bytes.iter().all(|&b| b == 0) {
            Ok(Previous::Open)
        } else {
            Ok(Previous::Block(BlockHash::try_from(slice_of_bytes)?))
        }
    }
}

impl FromStr for Previous {
    type Err = crate::Error;

    fn from_str(hex_string: &str) -> crate::Result<Self> {
        if hex_string == "0".repeat(64) {
            Ok(Previous::Open)
        } else {
            Ok(Previous::Block(BlockHash::from_str(hex_string)?))
        }
    }
}

/// A `Block` contains all block information needed for network and storage.
///
/// It has the fields of a state block, but can handle all block types.
///
/// When processing blocks from the network, this should be created after going through the
/// controller since certain fields such as "amount" won't be available immediately.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Block {
    #[serde(rename = "type")]
    block_type: BlockType,

    /// The cached hash of this block.
    hash: Option<BlockHash>,

    /// The account owner of this block.
    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    account: Public,

    /// Previous block hash on this account.
    previous: Previous,

    /// The representative this account is delegating to.
    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    representative: Public,

    /// The new balance of this account.
    balance: Rai,

    /// Link to either a send block, or a destination account.
    link: Link,

    /// The signed block's hash with the account's private key.
    signature: Option<Signature>,

    /// The proof of work applied to this block.
    work: Option<Work>,

    /// What level of trust do we have with this block?
    state: ValidationState,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ValidationState {
    Published,
    PresumedValid,
    Valid,
    SignatureFailed,
    WorkFailed,
}

impl Block {
    pub fn new(
        block_type: BlockType,
        account: Public,
        previous: Previous,
        representative: Public,
        balance: Rai,
        link: Link,
        state: ValidationState,
    ) -> Self {
        let mut new_block = Self {
            hash: None,
            block_type,
            account,
            previous,
            representative,
            balance,
            link,
            work: None,
            signature: None,
            state,
        };
        new_block.calc_hash().unwrap();
        new_block
    }

    pub fn from_open_block(open_block: &OpenBlock, previous: &Previous, balance: &Rai) -> Self {
        let mut b = Self::new(
            BlockType::Open,
            open_block.account.to_owned(),
            previous.to_owned(),
            open_block.representative.to_owned(),
            balance.to_owned(),
            Link::Source(open_block.source.to_owned()),
            ValidationState::Valid,
        );
        b.signature = open_block.signature.to_owned();
        b.work = open_block.work.to_owned();
        b
    }

    pub fn from_send_block(
        send_block: &SendBlock,
        account: &Public,
        representative: &Public,
    ) -> Self {
        let mut b = Self::new(
            BlockType::Send,
            account.to_owned(),
            Previous::Block(send_block.previous.to_owned()),
            representative.to_owned(),
            send_block.balance.to_owned(),
            Link::DestinationAccount(send_block.destination.to_owned()),
            ValidationState::Valid,
        );
        b.signature = send_block.signature.to_owned();
        b.work = send_block.work.to_owned();
        b
    }

    pub fn from_state_block(state_block: &StateBlock) -> Self {
        let mut b = Self::new(
            BlockType::State,
            state_block.account.to_owned(),
            state_block.previous.to_owned(),
            state_block.representative.to_owned(),
            state_block.balance.to_owned(),
            state_block.link.to_owned(),
            ValidationState::Valid,
        );
        b.signature = state_block.signature.to_owned();
        b.work = state_block.work.to_owned();
        b
    }

    pub fn hash(&self) -> anyhow::Result<&BlockHash> {
        match &self.hash {
            Some(block_hash) => Ok(&block_hash),
            None => Err(anyhow!("Block not hashable!")),
        }
    }

    /// Generates the hash for this block.
    /// Will be None if block type is Invalid or NotABlock
    // TODO: Can this ever fail?
    fn calc_hash(&mut self) -> anyhow::Result<()> {
        let context = || format!("Calculating hash for {:?}", &self);

        let hash_result = match &self.block_type() {
            BlockType::Open => hash_block(&[
                self.source().with_context(context)?.as_bytes(),
                self.representative.as_bytes(),
                self.account.as_bytes(),
            ]),
            BlockType::Send => hash_block(&[
                self.previous.to_bytes().as_slice(),
                self.destination().with_context(context)?.as_bytes(),
                self.balance.to_vec().as_slice(),
            ]),
            BlockType::Change => hash_block(&[
                self.previous.to_bytes().as_slice(),
                self.representative.as_bytes(),
            ]),
            BlockType::Receive => hash_block(&[
                self.previous.to_bytes().as_slice(),
                self.source().with_context(context)?.as_bytes(),
            ]),
            BlockType::State => {
                // TODO: check if epoch is *always* a state block
                let mut preamble = [0u8; 32];
                preamble[31] = BlockType::State as u8;

                hash_block(&[
                    &preamble,
                    self.account.as_bytes(),
                    self.previous.to_bytes().as_slice(),
                    self.representative.as_bytes(),
                    self.balance.to_vec().as_slice(),
                    self.link.as_bytes(),
                ])
            }
            _ => return Err(anyhow!("Block not hashable")),
        };

        self.hash = Some(hash_result);
        Ok(())
    }

    pub fn block_type(&self) -> &BlockType {
        &self.block_type
    }

    pub fn work(&self) -> Option<&Work> {
        self.work.as_ref()
    }

    pub fn set_work(&mut self, work: Work) {
        self.work = Some(work);
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn set_signature(&mut self, signature: Signature) {
        self.signature = Some(signature);
    }

    pub fn account(&self) -> &Public {
        &self.account
    }

    pub fn representative(&self) -> &Public {
        &self.representative
    }

    pub fn is_genesis(&self, network: &Network) -> anyhow::Result<bool> {
        Ok(&network.genesis_hash() == self.hash()?)
    }

    pub fn verify_signature(&self, account: &Public) -> anyhow::Result<()> {
        let hash = self.hash()?;
        let signature = self.signature().ok_or(anyhow!("Signature missing"))?;
        Ok(account
            .verify(hash.as_bytes(), signature)
            .context("Verify block")?)
    }

    pub fn sign(&mut self, private: Private) -> anyhow::Result<()> {
        let hash = self.hash()?;
        let signature = private.sign(hash.as_bytes())?;
        self.set_signature(signature);
        Ok(())
    }

    pub fn balance(&self) -> &Rai {
        &self.balance
    }

    pub fn previous(&self) -> &Previous {
        &self.previous
    }

    /// For an open or recv block, get the sender's block hash, otherwise Err.
    pub fn source(&self) -> anyhow::Result<&BlockHash> {
        if self.block_type != BlockType::Open {
            return Err(anyhow!(
                "Source requested for a {:?} block",
                self.block_type
            ));
        }

        if let Link::Source(hash) = &self.link {
            Ok(&hash)
        } else {
            Err(anyhow!(
                "source requested for {:?} but the link is incorrect",
                self
            ))
        }
    }

    /// For a send block, the destination account being sent to.
    pub fn destination(&self) -> anyhow::Result<&Public> {
        if self.block_type != BlockType::Send {
            return Err(anyhow!(
                "Destination requested for a {:?} block: {:?}",
                self.block_type,
                self
            ));
        }

        if let Link::DestinationAccount(account) = &self.link {
            Ok(&account)
        } else {
            Err(anyhow!(
                "destination requested for {:?} but the link is incorrect",
                self
            ))
        }
    }
}

pub fn hash_block(parts: &[&[u8]]) -> BlockHash {
    let mut v = Vec::new(); // TODO: with_capacity
    for b in parts {
        v.extend_from_slice(b);
    }
    // This unwrap should never fail because blake2b always returns a valid hash.
    BlockHash::try_from(blake2b(BlockHash::LEN, &v).as_ref()).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::blocks::{Block, BlockHash, Link, Previous, StateBlock};
    use crate::network::Network;
    use crate::{Public, Rai};
    use std::str::FromStr;

    #[test]
    fn json() {
        let genesis = Network::Live.genesis_block();
        let a = serde_json::to_string_pretty(&genesis).unwrap();
        dbg!(&a);
        assert!(a.contains(r#"type": "open""#));
        assert!(a.contains(r#"link": "E8"#));
        assert!(a.contains(r#"representative": "nano_3t"#));
        assert!(a.contains(r#"account": "nano_3t"#));
        assert!(a.contains(r#"work": "62F"#));
        assert!(a.contains(r#"signature": "9F"#));
    }

    fn test_state_block() -> StateBlock {
        let source = Link::Source(
            BlockHash::from_str("570EDFC56651FBBC9AEFE5B0769DBD210614A0C0E6962F5CA0EA2FFF4C08A4B0")
                .unwrap(),
        );
        let account =
            Public::from_str("570EDFC56651FBBC9AEFE5B0769DBD210614A0C0E6962F5CA0EA2FFF4C08A4B0")
                .unwrap();
        let representative =
            Public::from_str("7194452B7997A9F5ABB2F434DB010CA18B5A2715D141F9CFA64A296B3EB4DCCD")
                .unwrap();

        StateBlock::new(account, Previous::Open, representative, Rai(500), source)
    }

    #[test]
    fn round_trip_state_block() {
        let state_block_0 = test_state_block();
        let state_block_1 = StateBlock::from(Block::from_state_block(&state_block_0));
        assert_eq!(state_block_0, state_block_1)
    }

    #[test]
    fn round_trip_state_block2() {
        let state_block = test_state_block();
        let block_0 = Block::from_state_block(&state_block);
        let block_1 = Block::from_state_block(&StateBlock::from(block_0.clone()));
        assert_eq!(block_0, block_1)
    }
}
