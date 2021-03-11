use crate::cli::StringOrStdin;
use crate::{Language, MnemonicType};
use anyhow::anyhow;
use clap::Clap;
use std::str::FromStr;

#[derive(Clap)]
pub struct Phrase {
    #[clap(subcommand)]
    command: Command,
}

impl Phrase {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Random(x) => x.handle(),
            Command::Private(x) => x.handle(),
            Command::Public(x) => x.handle(),
            Command::Address(x) => x.handle(),
        }
    }
}

pub struct WrappedLanguage(pub Language);

impl FromStr for WrappedLanguage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let language = Language::from_language_code(s).ok_or(anyhow!(
            "Possible language codes are en, zh-hans, zh-hant, fr, it, ja, ko, es"
        ))?;
        Ok(WrappedLanguage(language))
    }
}

pub struct WrappedMnemonicType(MnemonicType);

impl FromStr for WrappedMnemonicType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = usize::from_str(s)?;
        let mt = MnemonicType::for_word_count(c)?;
        Ok(Self(mt))
    }
}

#[derive(Clap)]
pub enum Command {
    Random(Random),
    Private(Private),
    Public(Public),
    Address(Address),
}

#[derive(Clap)]
pub struct Random {
    #[clap(short, long, default_value = "24")]
    words: WrappedMnemonicType,

    #[clap(short, long, default_value = "en")]
    language: WrappedLanguage,
}

impl Random {
    pub fn handle(&self) -> anyhow::Result<()> {
        println!("{}", crate::Phrase::random(self.words.0, self.language.0));
        Ok(())
    }
}

#[derive(Clap)]
pub struct FromPhraseOpts {
    // Keep this as String because we need `phrase_opts` to work out how to convert into a Phrase.
    words: StringOrStdin<String>,

    #[clap(short, long, default_value = "en")]
    language: WrappedLanguage,

    #[clap(short, long, default_value = "0")]
    account: u32,

    // I tried using default_value = "" but clap still complained about the field being required.
    #[clap(short, long)]
    passphrase: Option<String>,
}

impl FromPhraseOpts {
    pub fn to_private(&self) -> anyhow::Result<crate::Private> {
        let words = self.words.to_owned().resolve().unwrap();
        let phrase = crate::Phrase::from_words(self.language.0, words.as_str())?;
        let private = phrase.to_private(
            self.account.to_owned(),
            self.passphrase.as_ref().unwrap_or(&"".to_string()).as_str(),
        )?;
        Ok(private)
    }
}

/// Convert a phrase to a private key.
#[derive(Clap)]
pub struct Private {
    #[clap(flatten)]
    opts: FromPhraseOpts,
}

impl Private {
    pub fn handle(&self) -> anyhow::Result<()> {
        let private = self.opts.to_private()?;
        println!("{}", private);
        Ok(())
    }
}

/// Convert a phrase to a public key.
#[derive(Clap)]
pub struct Public {
    #[clap(flatten)]
    opts: FromPhraseOpts,
}

impl Public {
    pub fn handle(&self) -> anyhow::Result<()> {
        let public = self.opts.to_private()?.to_public();
        println!("{}", public);
        Ok(())
    }
}

/// Convert a phrase to an address.
#[derive(Clap)]
pub struct Address {
    #[clap(flatten)]
    opts: FromPhraseOpts,
}

impl Address {
    pub fn handle(&self) -> anyhow::Result<()> {
        let address = self.opts.to_private()?.to_public().to_address();
        println!("{}", address);
        Ok(())
    }
}
