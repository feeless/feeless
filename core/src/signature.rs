use crate::encoding::hex_formatter;
use anyhow::anyhow;
use std::convert::TryFrom;

pub struct Signature([u8; Signature::LEN]);

impl Signature {
    pub const LEN: usize = 64;

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn internal(&self) -> ed25519_dalek::Signature {
        ed25519_dalek::Signature::new(self.0)
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
        if value.len() != Self::LEN {
            return Err(anyhow!(
                "Invalid length: {}, expecting: {}",
                value.len(),
                Self::LEN
            ));
        }

        let mut s = Signature([0u8; Self::LEN]);
        s.0.copy_from_slice(value);
        Ok(s)
    }
}
