use crate::state::State;
use crate::wire::Wire;
use feeless::{expect_len, BlockHash};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct HandleConfirmReq(BlockHash, BlockHash);

impl HandleConfirmReq {
    pub const LEN: usize = BlockHash::LEN * 2;
}

impl Wire for HandleConfirmReq {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(state: &State, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        expect_len(data.len(), HandleConfirmReq::LEN, "Handle confirmation req")?;
        Ok(Self(
            BlockHash::try_from(&data[0..BlockHash::LEN])?,
            BlockHash::try_from(&data[BlockHash::LEN..])?,
        ))
    }

    fn len() -> usize {
        BlockHash::LEN * 2
    }
}
