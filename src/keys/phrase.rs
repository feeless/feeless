//! BIP39 and BIP44 mnemonic seed phrase.
use crate::encoding::to_hex;
use crate::Error;
use crate::Private;
use bip39::Mnemonic;
pub use bip39::MnemonicType;
use ed25519_dalek_bip32::{DerivationPath, ExtendedSecretKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

static LANGUAGES: &str = "en, zh-hans, zh-hant, fr, it, ja, ko, es";

/// The language the phrase is in.
///
/// This is copied from [bip39::Language] because I need it to be Serialize/Deserialize. It should
/// act like the [crate::bip39] implementation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    English,
    ChineseSimplified,
    ChineseTraditional,
    French,
    Italian,
    Japanese,
    Korean,
    Spanish,
}

impl Language {
    pub fn from_language_code(language_code: &str) -> Option<Self> {
        bip39::Language::from_language_code(language_code).map(|x| x.into())
    }
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let language =
            Language::from_language_code(s).ok_or(Error::LanguageError(LANGUAGES.to_string()))?;
        Ok(language)
    }
}

impl From<bip39::Language> for Language {
    fn from(lang: bip39::Language) -> Self {
        match lang {
            bip39::Language::English => Language::English,
            bip39::Language::ChineseSimplified => Language::ChineseSimplified,
            bip39::Language::ChineseTraditional => Language::ChineseTraditional,
            bip39::Language::French => Language::French,
            bip39::Language::Italian => Language::Italian,
            bip39::Language::Japanese => Language::Japanese,
            bip39::Language::Korean => Language::Korean,
            bip39::Language::Spanish => Language::Spanish,
        }
    }
}

impl Into<bip39::Language> for Language {
    fn into(self) -> bip39::Language {
        match self {
            Language::English => bip39::Language::English,
            Language::ChineseSimplified => bip39::Language::ChineseSimplified,
            Language::ChineseTraditional => bip39::Language::ChineseTraditional,
            Language::French => bip39::Language::French,
            Language::Italian => bip39::Language::Italian,
            Language::Japanese => bip39::Language::Japanese,
            Language::Korean => bip39::Language::Korean,
            Language::Spanish => bip39::Language::Spanish,
        }
    }
}

/// A wrapper for Entropy so it can be serialized as hex, and have its own type instead of Vec<u8>.
// TODO: This should probably "act" more like the other [u8] structs.
#[derive(Debug, Clone)]
struct Entropy(Vec<u8>);

impl Serialize for Entropy {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for Entropy {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Self(
            hex::decode(s.as_bytes()).map_err(serde::de::Error::custom)?,
        ))
    }
}

/// BIP39 and BIP44 mnemonic seed phrase that can generate keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phrase {
    language: Language,
    entropy: Entropy,
}

impl Phrase {
    pub fn random(len: MnemonicType, language: Language) -> Self {
        let m = Mnemonic::new(len, language.to_owned().into());
        Self {
            entropy: Entropy(m.entropy().to_vec()),
            language: language.into(),
        }
    }

    pub fn to_mnemonic(&self) -> Result<Mnemonic, Error> {
        Ok(Mnemonic::from_entropy(
            &self.entropy.0,
            self.language.to_owned().into(),
        )?)
    }

    pub fn to_bip39_seed(&self, passphrase: &str) -> Result<bip39::Seed, Error> {
        Ok(bip39::Seed::new(&self.to_mnemonic()?, passphrase))
    }

    pub fn to_bip32_ext_key(
        &self,
        account: u32,
        passphrase: &str,
    ) -> Result<ExtendedSecretKey, Error> {
        let bip39_seed = self.to_bip39_seed(passphrase)?;
        let key = ExtendedSecretKey::from_seed(bip39_seed.as_bytes())?;
        let path = format!("m/44'/165'/{}'", account);
        let path: DerivationPath = path.parse().unwrap();
        let derived = key.derive(&path)?;

        Ok(derived)
    }

    pub fn to_private(&self, account: u32, passphrase: &str) -> Result<Private, Error> {
        let ext_key = self.to_bip32_ext_key(account, passphrase)?;
        let bip39_seed = ext_key.secret_key.as_ref();
        Ok(Private::try_from(bip39_seed)?)
    }

    pub fn from_words(language: Language, words: &str) -> Result<Self, Error> {
        let m = Mnemonic::from_phrase(words, language.to_owned().into())?;
        Ok(Self {
            language,
            entropy: Entropy(m.entropy().to_vec()),
        })
    }
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: remove unwrap
        let mnemonic = self.to_mnemonic().unwrap();
        let p = mnemonic.phrase();
        write!(f, "{}", &p)
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

        let address = private.to_public().unwrap().to_address();
        assert_eq!(
            address.to_string(),
            "nano_1pu7p5n3ghq1i1p4rhmek41f5add1uh34xpb94nkbxe8g4a6x1p69emk8y1d"
        );
    }
}
