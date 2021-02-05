use crate::expect_len;
use std::convert::TryFrom;

pub struct BlockHash([u8; BlockHash::LEN]);

impl BlockHash {
    const LEN: usize = 32;
}

impl TryFrom<&[u8]> for BlockHash {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Block hash")?;
        unimplemented!()
    }
}
