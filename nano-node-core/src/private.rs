use crate::Public;
use anyhow::anyhow;
use ed25519_dalek::SecretKey;
use std::convert::TryFrom;

pub const PRIVATE_KEY_BYTES: usize = 32;

pub struct Private(SecretKey);

impl Private {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_public(&self) -> Public {
        Public::from(self)
    }
}

impl TryFrom<&[u8]> for Private {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != PRIVATE_KEY_BYTES {
            return Err(anyhow!(
                "Private key is the wrong length: {} Expected: {}",
                bytes.len(),
                PRIVATE_KEY_BYTES
            ));
        }

        Ok(Self(SecretKey::from_bytes(bytes)?))
    }
}

impl std::fmt::UpperHex for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::fmt_hex(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}
