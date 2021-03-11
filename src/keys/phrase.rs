use crate::Private;
use anyhow::anyhow;
pub use bip39::Language;
use bip39::Mnemonic;
pub use bip39::MnemonicType;
use ed25519_dalek_bip32::{DerivationPath, ExtendedSecretKey};

use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Phrase(Mnemonic);

impl Phrase {
    pub fn random(len: MnemonicType, language: Language) -> Self {
        Self(Mnemonic::new(len, language))
    }

    pub fn to_bip39_seed(&self, passphrase: &str) -> anyhow::Result<bip39::Seed> {
        Ok(bip39::Seed::new(&self.0, passphrase))
    }

    pub fn to_bip32_ext_key(
        &self,
        account: u32,
        passphrase: &str,
    ) -> anyhow::Result<ExtendedSecretKey> {
        let bip39_seed = self.to_bip39_seed(passphrase)?;
        let key = ExtendedSecretKey::from_seed(bip39_seed.as_bytes())
            .map_err(|e| anyhow!("Extended secret key from BIP39 seed: {:?}", e))?;
        let path = format!("m/44'/165'/{}'", account);
        let path: DerivationPath = path.parse().unwrap();
        let derived = key
            .derive(&path)
            .map_err(|e| anyhow!("Deriving from bip39 seed to private key: {:?}", e))?;
        Ok(derived)
    }

    pub fn to_private(&self, account: u32, passphrase: &str) -> anyhow::Result<Private> {
        let ext_key = self.to_bip32_ext_key(account, passphrase)?;
        let bip39_seed = ext_key.secret_key.as_ref();
        Ok(Private::try_from(bip39_seed)?)
    }

    pub fn from_words(language: Language, words: &str) -> anyhow::Result<Self> {
        Ok(Self(Mnemonic::from_phrase(words, language)?))
    }
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.phrase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        // Example taken from:
        // https://docs.nano.org/integration-guides/key-management/#mnemonic-seed
        let phrase = Phrase::from_words(
            Language::English,
            "edge defense waste choose enrich upon flee junk siren film clown finish \
            luggage leader kid quick brick print evidence swap drill paddle truly occur",
        )
        .unwrap();

        let bip39_seed = phrase.to_bip39_seed("some password").unwrap();
        assert_eq!(
            format!("{:X}", bip39_seed),
            "0DC285FDE768F7FF29B66CE7252D56ED92FE003B605907F7A4F683C3DC8586D3\
            4A914D3C71FC099BB38EE4A59E5B081A3497B7A323E90CC68F67B5837690310C"
        );

        let private = phrase.to_private(0, "some password").unwrap();
        assert_eq!(
            format!("{:0X}", private),
            "3BE4FC2EF3F3B7374E6FC4FB6E7BB153F8A2998B3B3DAB50853EABE128024143"
        );

        let address = private.to_public().to_address();
        assert_eq!(
            address.to_string(),
            "nano_1pu7p5n3ghq1i1p4rhmek41f5add1uh34xpb94nkbxe8g4a6x1p69emk8y1d"
        );
    }
}
