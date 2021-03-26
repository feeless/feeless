use crate::encoding::{deserialize_from_str, hex_formatter};
use crate::{expect_len, to_hex};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::str::FromStr;
use crate::errors::FeelessError;

/// A ed25519+blake2 signature that can be generated with [Private](crate::Private) and
/// checked with [Public](crate::Public).
#[derive(Clone, PartialEq, Eq)]
pub struct Signature([u8; Signature::LEN]);

impl Signature {
    pub(crate) const LEN: usize = 64;

    pub(crate) fn zero() -> Self {
        Self([0u8; Signature::LEN])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn internal(&self) -> ed25519_dalek::Signature {
        ed25519_dalek::Signature::new(self.0)
    }
}

impl FromStr for Signature {
    type Err = FeelessError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Signature::try_from(hex::decode(s.as_bytes())
            .map_err(|e| FeelessError::FromHexError {
                msg: String::from("Decoding signature"),
                source: e,
            })?
            .as_slice())
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
        deserialize_from_str(deserializer)
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        hex_formatter(f, self.0.as_ref())
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = FeelessError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Signature")?;

        let mut s = Signature::zero();
        s.0.copy_from_slice(value);
        Ok(s)
    }
}
