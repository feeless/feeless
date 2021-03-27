use crate::encoding::{blake2b, deserialize_from_string};
use crate::{expect_len, to_hex, Private};

use crate::Error;
use bytes::{BufMut, BytesMut};
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

/// 256 bit seed used to derive multiple addresses.
///
/// See https://docs.nano.org/integration-guides/the-basics/#seed for details.
#[derive(Clone, PartialEq)]
pub struct Seed(pub [u8; Seed::LEN]);

impl Seed {
    const LEN: usize = 32;

    pub fn zero() -> Self {
        Self([0; Self::LEN])
    }

    /// Generate a secure random seed.
    pub fn random() -> Self {
        let mut seed = Seed::zero();
        rand::thread_rng().fill_bytes(&mut seed.0);
        seed
    }

    /// Derive a private key from the seed with an index.
    ///
    /// https://docs.nano.org/integration-guides/the-basics/#seed
    pub fn derive(&self, index: u32) -> Private {
        let mut buf = BytesMut::with_capacity(Self::LEN + 4); // seed + index
        buf.put(self.0.as_ref());
        buf.put_u32(index);

        let result = blake2b(Self::LEN, &buf);

        // Expect this to work all the time because it's coming from known correct types.
        Private::try_from(result.as_ref()).expect("conversion from seed")
    }
}

impl FromStr for Seed {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        expect_len(s.len(), Seed::LEN * 2, "Seed")?;
        let mut seed = Seed::zero();
        hex::decode_to_slice(s, &mut seed.0).map_err(|e| Error::FromHexError {
            msg: String::from("Decoding seed"),
            source: e,
        })?;
        Ok(seed)
    }
}

impl TryFrom<&[u8]> for Seed {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Seed::LEN, "Seed")?;
        let mut seed = Seed::zero();
        seed.0.copy_from_slice(value);
        Ok(seed)
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0)
    }
}

impl Debug for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0)
    }
}

impl Serialize for Seed {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_from_string(deserializer)
    }
}
