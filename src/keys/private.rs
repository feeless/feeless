use crate::{expect_len, hexify, Address, Error, Public, Signature};
use ed25519_dalek::ed25519::signature::Signature as InternalSignature;
use ed25519_dalek::ExpandedSecretKey;
use rand::RngCore;
use std::convert::TryFrom;
use std::str::FromStr;

/// 256 bit private key which can generate a public key.
#[derive(Clone)]
pub struct Private([u8; Private::LEN]);

hexify!(Private, "private key");

impl Private {
    pub(crate) const LEN: usize = 32;

    pub fn random() -> Self {
        let mut private = Private::zero();
        rand::thread_rng().fill_bytes(&mut private.0);
        private
    }

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
    pub fn to_public(&self) -> Result<Public, Error> {
        Ok(Public::from(self.internal_public()?))
    }

    pub(crate) fn internal_public(&self) -> Result<ed25519_dalek::PublicKey, Error> {
        let dalek = self.to_ed25519_dalek()?;
        Ok(ed25519_dalek::PublicKey::from(&dalek))
    }

    pub fn to_address(&self) -> Result<Address, Error> {
        Ok(self.to_public()?.to_address())
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        let dalek = self.to_ed25519_dalek()?;
        let expanded_secret = ExpandedSecretKey::from(&dalek);
        let internal_signed = expanded_secret.sign(message, &self.internal_public()?);
        Signature::try_from(internal_signed.as_bytes())
    }

    // Not public because we don't want users to accidentally generate this key.
    fn zero() -> Self {
        Self([0u8; 32])
    }

    fn to_ed25519_dalek(&self) -> Result<ed25519_dalek::SecretKey, Error> {
        Ok(
            ed25519_dalek::SecretKey::from_bytes(&self.0).map_err(|e| Error::SignatureError {
                msg: String::from("Converting to SecretKey"),
                source: e,
            })?,
        )
    }

    pub fn as_hex(&self) -> String {
        to_hex(self.0.as_ref())
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", &self)
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
