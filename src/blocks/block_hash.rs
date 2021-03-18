use crate::encoding::{deserialize_from_str, hex_formatter};
use crate::{expect_len, to_hex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BlockHash([u8; BlockHash::LEN]);

impl BlockHash {
    pub const LEN: usize = 32;

    pub fn zero() -> Self {
        Self([0u8; BlockHash::LEN])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl FromStr for BlockHash {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BlockHash::try_from(hex::decode(s.as_bytes())?.as_slice())
    }
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

impl std::fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockHash(")?;
        hex_formatter(f, &self.0)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl std::fmt::UpperHex for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        hex_formatter(f, &self.0.as_ref())
    }
}

impl Serialize for BlockHash {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for BlockHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_from_str(deserializer)
    }
}
