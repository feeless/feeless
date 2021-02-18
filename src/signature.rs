use crate::encoding::{deserialize_hex, hex_formatter, FromHex};
use crate::{expect_len, to_hex};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;

#[derive(Clone, PartialEq, Eq)]
pub struct Signature([u8; Signature::LEN]);

impl Signature {
    pub const LEN: usize = 64;

    pub fn zero() -> Self {
        Self([0u8; Signature::LEN])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn internal(&self) -> ed25519_dalek::Signature {
        ed25519_dalek::Signature::new(self.0)
    }
}

impl FromHex for Signature {
    fn from_hex(s: &str) -> anyhow::Result<Self> {
        Signature::try_from(hex::decode(s.as_bytes())?.as_slice())
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_hex(deserializer)
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        hex_formatter(f, self.0.as_ref())
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Signature")?;

        let mut s = Signature::zero();
        s.0.copy_from_slice(value);
        Ok(s)
    }
}
