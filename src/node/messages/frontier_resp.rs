use crate::blocks::BlockHash;
use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::Public;
use anyhow::Context;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct FrontierResp {
    account: Public,
    frontier_hash: BlockHash,
}

impl FrontierResp {
    pub const LEN: usize = Public::LEN + BlockHash::LEN;
}

impl Wire for FrontierResp {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        debug_assert!(header.is_none());
        let context = || format!("Deserialize frontier response");
        let mut bytes = Bytes::new(data);

        let account = bytes.slice(Public::LEN).with_context(context)?;
        let account = Public::try_from(account).with_context(context)?;

        let slice = bytes.slice(BlockHash::LEN).context("Slice FrontierHash")?;
        let frontier_hash = BlockHash::try_from(slice).context("Decode FrontierHash")?;

        Ok(Self {
            account,
            frontier_hash,
        })
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(Self::LEN)
    }
}
