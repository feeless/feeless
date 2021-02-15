use crate::encoding::hex_formatter;
use crate::{encoding, expect_len, to_hex, Address, Signature};
use anyhow::Context;
use bitvec::prelude::*;
use ed25519_dalek::Verifier;
use std::convert::TryFrom;
use std::iter::FromIterator;

// TODO: Don't keep the key as a dalek newtype. Only the bytes, and lazily use the crate
// when needed.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Public(ed25519_dalek::PublicKey);

impl Public {
    pub const LEN: usize = 32;

    const ADDRESS_CHECKSUM_LEN: usize = 5;

    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        Ok(Self(
            ed25519_dalek::PublicKey::from_bytes(
                hex::decode(s.as_bytes())
                    .context("Decoding hex public key")?
                    .as_slice(),
            )
            .with_context(|| format!("Loading public key from hex: {}", &s))?,
        ))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_hex(&self) -> String {
        to_hex(self.0.as_ref())
    }

    pub fn to_address(&self) -> Address {
        Address::from(self)
    }

    // Public key -> blake2(5) -> nano_base_32
    pub fn checksum(&self) -> String {
        let result = encoding::blake2b(Self::ADDRESS_CHECKSUM_LEN, &self.as_bytes());
        let bits = BitVec::from_iter(result.iter().rev());
        encoding::encode_nano_base_32(&bits)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.0.verify(message, &signature.internal()).is_ok()
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
        expect_len(value.len(), Self::LEN, "Public key")?;
        Ok(Self(ed25519_dalek::PublicKey::from_bytes(value)?))
    }
}

impl From<ed25519_dalek::PublicKey> for Public {
    fn from(v: ed25519_dalek::PublicKey) -> Self {
        Self(v)
    }
}

impl std::fmt::UpperHex for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}

#[cfg(test)]
mod tests {
    use super::Public;
    use crate::Private;
    use std::convert::TryFrom;

    /// Example private -> public conversion:
    /// https://docs.nano.org/protocol-design/signing-hashing-and-key-derivation/#signing-algorithm-ed25519
    #[test]
    fn empty_private_to_public() {
        let private_key_bytes = [0; Private::LEN];
        let private = Private::try_from(private_key_bytes.as_ref()).unwrap();
        let public = private.to_public();
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
        assert_eq!(s, &Public::from_hex(&s).unwrap().as_hex());
    }
}
