use crate::{Public, Signature};
use anyhow::anyhow;
use ed25519_dalek::ed25519::signature::Signature as InternalSignature;
use ed25519_dalek::ExpandedSecretKey;
use std::convert::TryFrom;

pub const PRIVATE_KEY_BYTES: usize = 32;

pub struct Private(ed25519_dalek::SecretKey);

impl Private {
    pub fn to_public(&self) -> Public {
        Public::from(self.internal_public())
    }

    pub fn internal_public(&self) -> ed25519_dalek::PublicKey {
        ed25519_dalek::PublicKey::from(&self.0)
    }

    pub fn sign(&self, message: &[u8]) -> anyhow::Result<Signature> {
        let expanded_secret = ExpandedSecretKey::from(&self.0);
        let internal_signed = expanded_secret.sign(message, &self.internal_public());
        Ok(Signature::try_from(internal_signed.as_bytes())?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Seed;

    #[tokio::test]
    async fn signing() {
        let message = [1, 2, 3, 4, 5];
        let private = Seed::random().derive(0);
        let public = private.to_public();
        let signature = private.sign(&message).unwrap();
        assert!(public.verify(&message, &signature));
    }
}
