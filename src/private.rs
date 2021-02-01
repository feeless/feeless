use bitvec::macros::internal::core::convert::TryFrom;
use ed25519_dalek::SecretKey;

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
        crate::encoding::fmt_hex(f, &self.0.as_bytes().as_ref())
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}
