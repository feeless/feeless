use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{BlockHash, Public};
use anyhow::Context;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct FrontierResp {
    account: [u8; Public::LEN],
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
        let mut bytes = Bytes::new(data);

        // let account = bytes.sized_slice::<Public>.context("Slice account")?;
        // let account =
        //     Public::try_from(slice).with_context(|| format!("Decode account {:?}", &slice))?;
        let account =
            <[u8; Public::LEN]>::try_from(bytes.slice(Public::LEN)?).context("Decode account")?;
        // let account = [0u8; Public::LEN];

        // dbg!(2);
        let slice = bytes.slice(BlockHash::LEN).context("Slice FrontierHash")?;
        let frontier_hash = BlockHash::try_from(slice).context("Decode FrontierHash")?;
        // dbg!(3);

        Ok(Self {
            account,
            frontier_hash,
        })
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(Self::LEN)
    }
}
