#[cfg(feature = "node")]
use crate::node::Wire;

#[cfg(feature = "node")]
use crate::node::Header;

use crate::hexify;
use crate::Error;
use crate::{encoding, Address, Signature};
use bitvec::prelude::*;
use ed25519_dalek::Verifier;
use serde::{Deserialize, Deserializer, Serializer};
use std::iter::FromIterator;
use std::str::FromStr;

/// 256 bit public key which can be converted into an [Address](crate::Address) or verify a [Signature](crate::Signature).
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Public([u8; Public::LEN]);

hexify!(Public, "public key");

impl Public {
    pub const LEN: usize = 32;
    const ADDRESS_CHECKSUM_LEN: usize = 5;

    fn dalek_key(&self) -> Result<ed25519_dalek::PublicKey, Error> {
        Ok(
            ed25519_dalek::PublicKey::from_bytes(&self.0).map_err(|e| Error::SignatureError {
                msg: String::from("Converting to PublicKey"),
                source: e,
            })?,
        )
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

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), Error> {
        let result = self.dalek_key();

        match result {
            Ok(key) => {
                key.verify(message, &signature.internal())
                    .map_err(|e| Error::SignatureError {
                        msg: format!(
                            "Public verification failed: sig: {:?} message: {:?} key: {:?}",
                            signature, message, key
                        ),
                        source: e,
                    })
            }
            // We're returning false here because someone we can be given a bad public key,
            // but since we're not checking the key for how valid it is, only the signature,
            // we just say that it does not pass validation.
            _ => Err(Error::BadPublicKey),
        }
    }
}

impl From<ed25519_dalek::PublicKey> for Public {
    fn from(v: ed25519_dalek::PublicKey) -> Self {
        Self(*v.as_bytes())
    }
}

#[cfg(feature = "node")]
impl Wire for Public {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_header: Option<&Header>, _data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn len(_header: Option<&Header>) -> anyhow::Result<usize>
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
    Ok(Address::from_str(s)
        .map_err(serde::de::Error::custom)?
        .to_public())
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
