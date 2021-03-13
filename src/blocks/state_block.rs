#[cfg(feature = "node")]
use crate::node::header::Header;

#[cfg(feature = "node")]
use crate::node::wire::Wire;

use crate::blocks::BlockType;
use crate::bytes::Bytes;
use crate::{expect_len, Block, BlockHash, Public, Raw, Signature, Work};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateBlock {
    pub account: Public,
    pub previous: BlockHash,
    pub representative: Public,
    pub balance: Raw,
    pub link: Link,
    pub work: Option<Work>,
    pub signature: Option<Signature>,
}

impl StateBlock {
    pub const LEN: usize = 216;

    pub fn new(
        account: Public,
        previous: BlockHash,
        representative: Public,
        balance: Raw,
        link: Link,
    ) -> Self {
        Self {
            account,
            previous,
            representative,
            balance,
            link,
            work: None,
            signature: None,
        }
    }
}

#[cfg(feature = "node")]
impl Wire for StateBlock {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut data = Bytes::new(data);

        let account = Public::try_from(data.slice(Public::LEN)?)?;
        let previous = BlockHash::try_from(data.slice(BlockHash::LEN)?)?;
        let representative = Public::try_from(data.slice(Public::LEN)?)?;
        let balance = Raw::try_from(data.slice(Raw::LEN)?)?;

        let link_data = data.slice(Public::LEN)?;
        // We are unsure because we need to work out the previous balance of this account first.
        let link = Link::Unsure(<[u8; 32]>::try_from(link_data)?);

        let signature = Signature::try_from(data.slice(Signature::LEN)?)?;
        let work = Work::try_from(data.slice(Work::LEN)?)?;

        let mut block = Self::new(account, previous, representative, balance, link);
        block.signature = Some(signature);
        block.work = Some(work);
        Ok(block)
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error> {
        debug_assert!(header.is_some());
        let header = header.unwrap();
        debug_assert_eq!(header.ext().block_type()?, BlockType::State);

        Ok(StateBlock::LEN)
    }
}

#[cfg(test)]
mod tests {
    use crate::blocks::state_block::Link;
    use crate::{Address, Signature, Work};

    use super::StateBlock;
    use super::{Block, BlockHash, Raw};
    use std::str::FromStr;

    #[test]
    fn hash_a_real_state_block() {
        let account =
            Address::from_str("nano_34prihdxwz3u4ps8qjnn14p7ujyewkoxkwyxm3u665it8rg5rdqw84qrypzk")
                .unwrap()
                .to_public();
        let parent =
            BlockHash::from_str("7837C80964CAD551DEABE162C7FC4BB58688A0C6EB6D9907C0D2A7C74A33C7EB")
                .unwrap();
        let representative = account.clone();
        let balance = Raw::from_raw(2711469892748129430069222848295u128);
        let link = Link::Source(
            BlockHash::from_str("0399B19B022D260F3DDFBA26D0306D423F1890D3AE06136FAB16802D1F2B87A7")
                .unwrap(),
        );
        // Signature and work aren't hashed, but left them as the real data anyway.
        let signature = Signature::from_str("BCF9F123138355AE9E741912D319FF48E5FCCA39D9E5DD74411D32C69B1C7501A0BF001C45D4F68CB561B902A42711E6166B9018E76C50CC868EF2E32B78F200").unwrap();
        let work = Work::from_str("d4757052401b9e08").unwrap();

        let block = StateBlock::new(account, parent, representative, balance, link);
        let mut block = Block::from_state_block(&block);

        block.set_signature(signature);
        block.set_work(work);
        block.calc_hash().unwrap();

        assert_eq!(
            block.hash().unwrap(),
            &BlockHash::from_str(
                "6F050D3D0B19C2C206046AAE2D46661B57E1B7D890DE8398D203A025E29A4AD9"
            )
            .unwrap()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Link {
    /// For the change block type.
    Nothing,

    /// When we've received and decoded a block, but don't know what kind of block this is yet.
    Unsure([u8; Link::LEN]),

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

    pub fn unsure_from_str(s: &str) -> anyhow::Result<Self> {
        expect_len(s.len(), Self::LEN * 2, "Link")?;
        let mut slice = [0u8; Self::LEN];
        hex::decode_to_slice(s, &mut slice)?;
        Ok(Link::Unsure(slice))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Link::Nothing => &[0u8; Self::LEN],
            Link::Source(hash) => hash.as_bytes(),
            Link::DestinationAccount(key) => key.as_bytes(),
            Link::Unsure(b) => b.as_ref(),
        }
    }
}
