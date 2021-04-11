use crate::encoding::{deserialize_from_str, hex_formatter};
use crate::{expect_len, hexify, to_hex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BlockHash([u8; BlockHash::LEN]);

hexify!(BlockHash, "block hash");

impl BlockHash {
    pub const LEN: usize = 32;

    pub fn zero() -> Self {
        Self([0u8; BlockHash::LEN])
    }
}

impl std::fmt::Display for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}
