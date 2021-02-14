use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{BlockHash, Public};
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

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut bytes = Bytes::new(data);

        let account =
            Public::try_from(bytes.slice(Public::LEN).with_context("Public")?).context("Public")?;
        let frontier_hash =
            BlockHash::try_from(bytes.slice(BlockHash::LEN).with_context("FrontierHash")?)
                .context("FrontierHash")?;

        Ok(Self {
            account,
            frontier_hash,
        })
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(Self::LEN)
    }
}
