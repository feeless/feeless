use crate::hexify;

/// A ed25519+blake2 signature that can be generated with [Private](crate::Private) and
/// checked with [Public](crate::Public).
#[derive(Clone, PartialEq, Eq)]
pub struct Signature([u8; Signature::LEN]);

hexify!(Signature, "signature");

impl Signature {
    pub(crate) const LEN: usize = 64;

    pub(crate) fn zero() -> Self {
        Self([0u8; Signature::LEN])
    }

    pub(crate) fn internal(&self) -> ed25519_dalek::Signature {
        ed25519_dalek::Signature::new(self.0)
    }
}
