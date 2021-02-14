use crate::expect_len;
use std::cmp::Ordering;
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq)]
pub struct Difficulty(u64);

impl Difficulty {
    const LEN: usize = 8;
    const HEX_LEN: usize = Self::LEN * 2;

    pub fn new(v: u64) -> Self {
        Self(v)
    }

    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        expect_len(s.len(), Self::HEX_LEN, "Difficulty")?;
        let mut slice = [0u8; Self::LEN];
        hex::decode_to_slice(s, &mut slice)?;
        Self::from_be_slice(&slice)
    }

    pub fn from_fixed_slice(s: &[u8; Self::LEN]) -> anyhow::Result<Self> {
        Ok(Difficulty(u64::from_le_bytes(*s)))
    }

    // pub fn from_be_slice(s: &[u8]) -> anyhow::Result<Self> {
    //     let mut b = [0u8; Self::LEN];
    //     b.copy_from_slice(s);
    //     Ok(Difficulty(u64::from_be_bytes(b)))
    // }

    pub fn from_be_slice(s: &[u8]) -> anyhow::Result<Self> {
        let b = <[u8; Self::LEN]>::try_from(s)?;
        Ok(Difficulty(u64::from_be_bytes(b)))
    }

    pub fn from_le_slice(s: &[u8]) -> anyhow::Result<Self> {
        let mut b = [0u8; Self::LEN];
        b.copy_from_slice(s);
        Ok(Difficulty(u64::from_le_bytes(b)))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl PartialOrd for Difficulty {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert_eq!(
            Difficulty::from_hex("ffffffc000000000").unwrap().as_u64(),
            18446743798831644672u64
        );
    }
}
