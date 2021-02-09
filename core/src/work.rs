use crate::expect_len;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Work([u8; Work::LEN]);

impl Work {
    pub const LEN: usize = 8;

    pub fn zero() -> Self {
        Self([0u8; 8])
    }

    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        Work::try_from(hex::decode(s.as_bytes())?.as_slice())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Work {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Work")?;

        let mut s = Work::zero();
        s.0.copy_from_slice(value);
        Ok(s)
    }
}
