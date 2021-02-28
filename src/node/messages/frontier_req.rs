use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::Public;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct FrontierReq {
    start: Public,
    age: u32,
    count: u32,
}

impl FrontierReq {
    pub const LEN: usize = 40;
}

impl Wire for FrontierReq {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut bytes = Bytes::new(data);
        let start =
            Public::try_from(bytes.slice(Public::LEN)?).expect("frontier req deserializing start");

        let mut s32 = [0u8; 4];
        s32.copy_from_slice(bytes.slice(4)?);
        let age = u32::from_le_bytes(s32);
        s32.copy_from_slice(bytes.slice(4)?);
        let count = u32::from_le_bytes(s32);

        Ok(Self { start, age, count })
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(Self::LEN)
    }
}
