#[cfg(feature = "node")]
use crate::node::Header;

#[cfg(feature = "node")]
use crate::node::Wire;

use crate::blocks::{hash_block, Block, BlockHash, BlockType, Previous};
use crate::bytes::Bytes;
use crate::encoding::expect_len;
use crate::keys::public::{from_address, to_address};
use crate::{hexify, Error, Public, Rai, Result, Signature, Work};
use anyhow::anyhow;
use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use strum_macros::EnumString;

pub fn deserialize_to_unsure_link<'de, D>(
    deserializer: D,
) -> std::result::Result<Link, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
{
    let unsure = UnsureLink::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    Ok(Link::Unsure(unsure))
}

/// Not used within StateBlock yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Subtype {
    Send,
    Receive,
    Open,
    Change,
    Epoch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateBlock {
    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub account: Public,

    pub previous: Previous,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub representative: Public,

    pub balance: Rai,

    #[serde(deserialize_with = "deserialize_to_unsure_link")]
    pub link: Link,

    pub work: Option<Work>,

    pub signature: Option<Signature>,

    pub hash: BlockHash,

    #[serde(skip_serializing, skip_deserializing)] // only exists during processing
    amount: Option<Amount>,
}

impl StateBlock {
    pub const LEN: usize = 216;

    pub fn new(
        account: Public,
        previous: Previous,
        representative: Public,
        balance: Rai,
        link: Link,
    ) -> Self {
        let mut preamble = [0u8; 32];
        preamble[31] = BlockType::State as u8;

        let block_hash = hash_block(&[
            &preamble,
            account.as_bytes(),
            previous.to_bytes().as_slice(),
            representative.as_bytes(),
            balance.to_vec().as_slice(),
            link.as_bytes(),
        ]);
        Self {
            account,
            previous,
            representative,
            balance,
            link,
            work: None,
            signature: None,
            hash: block_hash,
            amount: None,
        }
    }

    pub fn set_link_type(&mut self, is_send: bool, amount: Rai) -> anyhow::Result<()> {
        match &self.link {
            Link::Nothing => {
                tracing::trace!("set_link_type likely called twice by mistake.");
                Ok(())
            }
            Link::Source(_) => {
                tracing::trace!("set_link_type likely called twice by mistake.");
                Ok(())
            }
            Link::DestinationAccount(_) => {
                tracing::trace!("set_link_type likely called twice by mistake.");
                Ok(())
            }
            Link::Unsure(unsure_link) => {
                if is_send {
                    self.link = Link::DestinationAccount(Public::try_from(unsure_link.as_bytes())?);
                    self.amount = Some(Amount(amount))
                } else {
                    let is_receive = !is_send && !unsure_link.is_all_zeros();
                    let is_change = !is_send && unsure_link.is_all_zeros();

                    if is_receive {
                        self.link = Link::Source(BlockHash::try_from(unsure_link.as_bytes())?);
                        self.amount = Some(Amount(amount))
                    } else if is_change {
                        debug_assert_eq!(amount, Rai(0));
                        self.link = Link::Nothing
                    }
                }
                Ok(())
            }
        }
    }

    pub fn verify_self_signature(&self) -> anyhow::Result<()> {
        let signature = self
            .signature
            .as_ref()
            .ok_or(anyhow!("Signature missing"))?;
        Ok(self
            .account
            .verify(self.hash.as_bytes(), signature)
            .context("Verify block")?)
    }
}

#[cfg(feature = "node")]
impl Wire for StateBlock {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut data = Bytes::new(data);

        let account = Public::try_from(data.slice(Public::LEN)?)?;
        let previous = Previous::try_from(data.slice(BlockHash::LEN)?)?;
        let representative = Public::try_from(data.slice(Public::LEN)?)?;
        let balance = Rai::try_from(data.slice(Rai::LEN)?)?;

        let link_data = data.slice(Public::LEN)?;
        // We are unsure because we need to work out the previous balance of this account first.
        let unsure = UnsureLink::try_from(link_data)?;
        let link = Link::Unsure(unsure);

        let signature = Signature::try_from(data.slice(Signature::LEN)?)?;
        let work = Work::try_from(data.slice(Work::LEN)?)?;

        let mut block = Self::new(account, previous, representative, balance, link);
        block.signature = Some(signature);
        block.work = Some(work);
        Ok(block)
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize> {
        debug_assert!(header.is_some());
        let header = header.unwrap();
        debug_assert_eq!(header.ext().block_type()?, BlockType::State);

        Ok(StateBlock::LEN)
    }
}

impl From<Block> for StateBlock {
    fn from(block: Block) -> Self {
        StateBlock::new(
            block.account,
            block.previous,
            block.representative,
            block.balance,
            block.link,
        )
    }
}

impl Display for StateBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let work_render = match &self.work {
            Some(work) => work.to_string(),
            None => "No work".to_string(),
        };
        let signature_render = match &self.signature {
            Some(signature) => signature.to_string(),
            None => "No signature".to_string(),
        };
        write!(
            f,
            "Block(Account: {}, Previous: {:?}, Balance: {}, Link: {:?}, Work: {}, Signature: {})",
            self.account, self.previous, self.balance, self.link, work_render, signature_render
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Rai;
    use super::StateBlock;
    use crate::blocks::state_block::{Amount, Link, UnsureLink};
    use crate::blocks::{Block, BlockHash, Previous};
    use crate::{Address, Public, Signature, Work};
    use std::str::FromStr;

    fn account_0() -> Public {
        Address::from_str("nano_34prihdxwz3u4ps8qjnn14p7ujyewkoxkwyxm3u665it8rg5rdqw84qrypzk")
            .unwrap()
            .to_public()
    }
    fn parent_0() -> Previous {
        Previous::from_str("7837C80964CAD551DEABE162C7FC4BB58688A0C6EB6D9907C0D2A7C74A33C7EB")
            .unwrap()
    }
    fn representative_0() -> Public {
        account_0()
    }
    fn balance_0() -> Rai {
        Rai::new(2711469892748129430069222848295u128)
    }

    #[test]
    fn hash_a_real_state_block() {
        let link = Link::Source(
            BlockHash::from_str("0399B19B022D260F3DDFBA26D0306D423F1890D3AE06136FAB16802D1F2B87A7")
                .unwrap(),
        );
        // Signature and work aren't hashed, but left them as the real data anyway.
        let signature = Signature::from_str("BCF9F123138355AE9E741912D319FF48E5FCCA39D9E5DD74411D32C69B1C7501A0BF001C45D4F68CB561B902A42711E6166B9018E76C50CC868EF2E32B78F200").unwrap();
        let work = Work::from_str("d4757052401b9e08").unwrap();

        let block = StateBlock::new(
            account_0(),
            parent_0(),
            representative_0(),
            balance_0(),
            link,
        );
        let mut block = Block::from_state_block(&block);

        block.set_signature(signature);
        block.set_work(work);

        assert_eq!(
            block.hash().unwrap(),
            &BlockHash::from_str(
                "6F050D3D0B19C2C206046AAE2D46661B57E1B7D890DE8398D203A025E29A4AD9"
            )
            .unwrap()
        )
    }

    #[test]
    fn set_destination_link() {
        let unsure_link = Link::Unsure(
            UnsureLink::from_str(
                "6B523BCB57B0997C808D89BA30F78BF5E4E7DAE880BFDC4179B537F0D8ED726E",
            )
            .unwrap(),
        );
        let destination_account =
            Public::from_str("6B523BCB57B0997C808D89BA30F78BF5E4E7DAE880BFDC4179B537F0D8ED726E")
                .unwrap();
        let mut state_block = StateBlock::new(
            account_0(),
            parent_0(),
            representative_0(),
            balance_0(),
            unsure_link,
        );

        state_block.set_link_type(true, Rai(200)).unwrap();

        assert_eq!(state_block.amount, Some(Amount(Rai(200))));
        assert_eq!(
            state_block.link,
            Link::DestinationAccount(destination_account)
        );
    }

    #[test]
    fn set_source_link() {
        let unsure_link = Link::Unsure(
            UnsureLink::from_str(
                "6B523BCB57B0997C808D89BA30F78BF5E4E7DAE880BFDC4179B537F0D8ED726E",
            )
            .unwrap(),
        );
        let source =
            BlockHash::from_str("6B523BCB57B0997C808D89BA30F78BF5E4E7DAE880BFDC4179B537F0D8ED726E")
                .unwrap();
        let mut state_block = StateBlock::new(
            account_0(),
            parent_0(),
            representative_0(),
            balance_0(),
            unsure_link,
        );

        state_block.set_link_type(false, Rai(200)).unwrap();

        assert_eq!(state_block.amount, Some(Amount(Rai(200))));
        assert_eq!(state_block.link, Link::Source(source));
    }

    #[test]
    fn set_nothing_link() {
        let unsure_link = Link::Unsure(UnsureLink([0u8; 32]));
        let mut state_block = StateBlock::new(
            account_0(),
            parent_0(),
            representative_0(),
            balance_0(),
            unsure_link,
        );

        state_block.set_link_type(false, Rai(0)).unwrap();

        assert_eq!(state_block.amount, None);
        assert_eq!(state_block.link, Link::Nothing);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UnsureLink([u8; Self::LEN]);

hexify!(UnsureLink, "link");

impl UnsureLink {
    pub(crate) const LEN: usize = Link::LEN;

    pub fn is_all_zeros(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

/// Represent nanos transferred between account in send and receive sub-blocks
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Amount(Rai);

/// Used in state block as a reference to either the previous block or a destination address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", untagged)]
pub enum Link {
    /// For the change block type.
    Nothing,

    /// When we've received and decoded a block, but don't know what kind of block this is yet.
    Unsure(UnsureLink),

    /// Reference the previous block, for receiving.
    Source(BlockHash),

    /// Send to a destination account.
    DestinationAccount(Public),
}

impl Link {
    pub const LEN: usize = 32;

    pub fn nothing() -> Self {
        Self::Nothing
    }

    pub fn unsure_from_str(s: &str) -> Result<Self> {
        expect_len(s.len(), Self::LEN * 2, "Link")?;
        let mut slice = [0u8; Self::LEN];
        hex::decode_to_slice(s, &mut slice).map_err(|source| Error::FromHexError {
            source,
            msg: "Decoding link hex".into(),
        })?;
        Ok(Link::Unsure(UnsureLink(slice)))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Link::Nothing => &[0u8; Self::LEN],
            Link::Source(hash) => hash.as_bytes(),
            Link::DestinationAccount(key) => key.as_bytes(),
            Link::Unsure(b) => b.as_bytes(),
        }
    }
}

impl FromStr for Link {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::unsure_from_str(s)
    }
}
