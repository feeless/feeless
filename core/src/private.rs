use crate::Public;
use anyhow::anyhow;
use std::convert::TryFrom;

pub const PRIVATE_KEY_BYTES: usize = 32;

pub struct Private(ed25519_dalek::SecretKey);

impl Private {
    pub fn to_public(&self) -> Public {
        Public::from(ed25519_dalek::PublicKey::from(&self.0))
    }
}

impl TryFrom<&[u8]> for Private {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != PRIVATE_KEY_BYTES {
            return Err(anyhow!(
                "Private key is the wrong length: Got: {} Expected: {}",
                bytes.len(),
                PRIVATE_KEY_BYTES
            ));
        }

        Ok(Self(ed25519_dalek::SecretKey::from_bytes(bytes)?))
    }
}

impl std::fmt::UpperHex for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}
