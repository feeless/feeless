use std::convert::TryFrom;
use blake2::{Blake2b, Digest};
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey};

pub struct Private(SecretKey);

impl Private {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<&[u8]> for Private {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(SecretKey::from_bytes(bytes)?))
    }
}

impl std::fmt::UpperHex for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::fmt_hex(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}

pub struct Public(PublicKey);

impl Public {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}


impl From<&Private> for Public {
    fn from(private_key: &Private) -> Self {
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

impl std::fmt::UpperHex for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::fmt_hex(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Example private -> public conversion:
        /// https://docs.nano.org/protocol-design/signing-hashing-and-key-derivation/#signing-algorithm-ed25519
    #[test]
    fn empty_private_to_public() {
        let private_key_bytes = [0; 32];
        let private = Private::try_from(private_key_bytes.as_ref()).unwrap();
        let public = Public::from(&private);
        assert_eq!(public.to_string(), "19D3D919475DEED4696B5D13018151D1AF88B2BD3BCFF048B45031C1F36D1858")
    }
}