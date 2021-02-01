use crate::{encoding, Address, Private};
use anyhow::anyhow;
use bitvec::prelude::*;
use blake2::{Blake2b, Digest};
use ed25519_dalek::{ExpandedSecretKey, PublicKey};
use std::convert::TryFrom;
use std::iter::FromIterator;

pub const PUBLIC_KEY_BYTES: usize = 32;

const ADDRESS_CHECKSUM_LEN: usize = 5;

#[derive(Debug, PartialEq)]
pub struct Public(PublicKey);

impl Public {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_address(&self) -> Address {
        Address::from(self)
    }

    // Public key -> blake2(5) -> nano_base_32
    pub(crate) fn checksum(&self) -> String {
        let result = encoding::blake2b(ADDRESS_CHECKSUM_LEN, &self.as_bytes());
        let bits = BitVec::from_iter(result.iter().rev());
        encoding::encode_nano_base_32(&bits)
    }
}

impl From<&Private> for Public {
    fn from(private_key: &Private) -> Self {
        // TODO: Check for length

        // This is modified from ed25519_dalek::PublicKey::from(secret_key: &SecretKey) so that
        // it can use Blake2b instead of SHA256.
        let mut h: Blake2b = Blake2b::new();
        let mut hash: [u8; 64] = [0u8; 64];
        let mut digest: [u8; 32] = [0u8; 32];

        h.update(private_key.as_bytes());
        hash.copy_from_slice(h.finalize().as_slice());
        digest.copy_from_slice(&hash[..32]);

        // Unwrap here because we expect this to work, given presumably any private key.
        let expanded = ExpandedSecretKey::from_bytes(&hash).unwrap();
        let public_key = PublicKey::from(&expanded);
        Self(public_key)
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != PUBLIC_KEY_BYTES {
            return Err(anyhow!(
                "Invalid length: {}, expecting: {}",
                value.len(),
                PUBLIC_KEY_BYTES
            ));
        }

        Ok(Self(PublicKey::from_bytes(value)?))
    }
}

impl std::fmt::UpperHex for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::fmt_hex(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}

#[cfg(test)]
mod tests {
    use crate::private::PRIVATE_KEY_BYTES;
    use crate::{Private, Public};
    use std::convert::TryFrom;

    /// Example private -> public conversion:
    /// https://docs.nano.org/protocol-design/signing-hashing-and-key-derivation/#signing-algorithm-ed25519
    #[test]
    fn empty_private_to_public() {
        let private_key_bytes = [0; PRIVATE_KEY_BYTES];
        let private = Private::try_from(private_key_bytes.as_ref()).unwrap();
        let public = Public::from(&private);
        assert_eq!(
            public.to_string(),
            "19D3D919475DEED4696B5D13018151D1AF88B2BD3BCFF048B45031C1F36D1858"
        )
    }
}
