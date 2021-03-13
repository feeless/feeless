#[cfg(feature = "node")]
use crate::node::wire::Wire;

#[cfg(feature = "node")]
use crate::node::header::Header;

use crate::encoding::deserialize_hex;
use crate::{encoding, expect_len, len_err_msg, to_hex, Address, Signature};
use anyhow::{anyhow, Context};
use bitvec::prelude::*;
use ed25519_dalek::Verifier;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::str::FromStr;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Public([u8; Public::LEN]);

impl Public {
    pub const LEN: usize = 32;
    const ADDRESS_CHECKSUM_LEN: usize = 5;

    fn dalek_key(&self) -> anyhow::Result<ed25519_dalek::PublicKey> {
        ed25519_dalek::PublicKey::from_bytes(&self.0).context("Loading dalek key")
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_hex(&self) -> String {
        to_hex(self.0.as_ref())
    }

    pub fn to_address(&self) -> Address {
        Address::from(self)
    }

    // Public key -> blake2(5) -> nano_base_32
    pub fn checksum(&self) -> String {
        let result = encoding::blake2b(Self::ADDRESS_CHECKSUM_LEN, &self.0);
        let bits = BitVec::from_iter(result.iter().rev());
        encoding::encode_nano_base_32(&bits)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> anyhow::Result<()> {
        let result = self
            .dalek_key()
            .with_context(|| format!("Verify {:?} with {:?}", message, signature));

        match result {
            Ok(key) => key.verify(message, &signature.internal()).with_context(|| {
                format!(
                    "Public verification failed: sig: {:?} message: {:?} key: {:?}",
                    signature, message, self
                )
            }),
            // We're returning false here because someone we can be given a bad public key,
            // but since we're not checking the key for how valid it is, only the signature,
            // we just say that it does not pass validation.
            Err(e) => Err(e.context("Bad public key, can not verify")),
        }
    }
}

impl FromStr for Public {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        expect_len(s.len(), Self::LEN * 2, "hex public key")?;
        let vec = hex::decode(s.as_bytes()).context("Decoding hex public key")?;
        let bytes = vec.as_slice();

        let x = <[u8; Self::LEN]>::try_from(bytes)
            .with_context(|| format!("Could not convert bytes into slice {:?}", bytes))?;
        Ok(Self(x))
    }
}

impl std::fmt::Debug for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Public({} {})",
            to_hex(self.0.as_ref()),
            self.to_address()
        )
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(<[u8; Self::LEN]>::try_from(value).with_context(
            || len_err_msg(value.len(), Self::LEN, "Public key"),
        )?))
    }
}

impl From<ed25519_dalek::PublicKey> for Public {
    fn from(v: ed25519_dalek::PublicKey) -> Self {
        Self(*v.as_bytes())
    }
}

impl std::fmt::UpperHex for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0)
    }
}

impl std::fmt::Display for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}

impl Serialize for Public {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for Public {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_hex(deserializer)
    }
}

#[cfg(feature = "node")]
impl Wire for Public {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        Ok(Self::LEN)
    }
}

/// A serde serializer that converts to an address instead of public key hexes.
///
/// Use with #[serde(serialize_with = "to_address")] on the field that needs it.
pub fn to_address<S>(public: &Public, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(public.to_address().to_string().as_str())
}

pub fn from_address<'de, D>(deserializer: D) -> Result<Public, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(Address::from_str(s).map_err(D::Error::custom)?.to_public())
}

#[cfg(test)]
mod tests {
    use super::Public;
    use crate::Private;
    use std::convert::TryFrom;
    use std::str::FromStr;

    /// Example private -> public conversion:
    /// https://docs.nano.org/protocol-design/signing-hashing-and-key-derivation/#signing-algorithm-ed25519
    #[test]
    fn empty_private_to_public() {
        let private_key_bytes = [0; Private::LEN];
        let private = Private::try_from(private_key_bytes.as_ref()).unwrap();
        let public = private.to_public().unwrap();
        // If the result is...
        // 3B6A27BCCEB6A42D62A3A8D02A6F0D73653215771DE243A63AC048A18B59DA29
        // ...it means we're using sha512 instead of blake2b for the hasher.
        assert_eq!(
            public.to_string(),
            "19D3D919475DEED4696B5D13018151D1AF88B2BD3BCFF048B45031C1F36D1858"
        )
    }

    #[test]
    fn hex() {
        let s = "19D3D919475DEED4696B5D13018151D1AF88B2BD3BCFF048B45031C1F36D1858";
        assert_eq!(s, &Public::from_str(&s).unwrap().as_hex());
    }
}
