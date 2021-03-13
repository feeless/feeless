use crate::{expect_len, Public, Signature};
use anyhow::Context;
use ed25519_dalek::ed25519::signature::Signature as InternalSignature;
use ed25519_dalek::ExpandedSecretKey;
use std::convert::TryFrom;
use std::str::FromStr;

/// 256 bit private key which can generate a public key.
#[derive(Clone)]
pub struct Private([u8; Private::LEN]);

impl Private {
    pub(crate) const LEN: usize = 32;

    /// Generate the public key for this private key.
    ///
    /// If you wish to convert this private key to a Nano address you will need to take another
    /// step:
    /// ```
    /// use feeless::Private;
    /// use std::str::FromStr;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let s = "0000000000000000000000000000000000000000000000000000000000000000";
    /// let address = Private::from_str(s)?.to_public()?.to_address();
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_public(&self) -> anyhow::Result<Public> {
        Ok(Public::from(self.internal_public()?))
    }

    pub fn internal_public(&self) -> anyhow::Result<ed25519_dalek::PublicKey> {
        let dalek = self.to_ed25519_dalek()?;
        Ok(ed25519_dalek::PublicKey::from(&dalek))
    }

    pub fn sign(&self, message: &[u8]) -> anyhow::Result<Signature> {
        let dalek = self.to_ed25519_dalek()?;
        let expanded_secret = ExpandedSecretKey::from(&dalek);
        let internal_signed = expanded_secret.sign(message, &self.internal_public()?);
        Signature::try_from(internal_signed.as_bytes())
    }

    fn to_ed25519_dalek(&self) -> anyhow::Result<ed25519_dalek::SecretKey> {
        Ok(ed25519_dalek::SecretKey::from_bytes(&self.0)?)
    }
}

impl TryFrom<&[u8]> for Private {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        expect_len(bytes.len(), Private::LEN, "Private key")?;
        let x = <[u8; Self::LEN]>::try_from(bytes)?;
        Ok(Self(x))
    }
}

impl std::fmt::UpperHex for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0)
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
    }
}

impl FromStr for Private {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        expect_len(s.len(), Self::LEN * 2, "hex private key")?;
        let vec = hex::decode(s.as_bytes()).context("Decoding hex public key")?;
        let bytes = vec.as_slice();
        Self::try_from(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::Seed;

    #[tokio::test]
    async fn signing() {
        let message = [1, 2, 3, 4, 5];
        let private = Seed::random().derive(0);
        let public = private.to_public().unwrap();
        let signature = private.sign(&message).unwrap();
        assert!(public.verify(&message, &signature).is_ok());
    }
}
