use crate::encoding::hex_formatter;
use crate::expect_len;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct BlockHash([u8; BlockHash::LEN]);

impl BlockHash {
    pub const LEN: usize = 32;
}

impl TryFrom<&[u8]> for BlockHash {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Block hash")?;

        let mut bh = BlockHash([0u8; Self::LEN]);
        bh.0.copy_from_slice(&value);
        Ok(bh)
    }
}

impl std::fmt::Display for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}

impl std::fmt::UpperHex for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        hex_formatter(f, &self.0.as_ref())
    }
}
