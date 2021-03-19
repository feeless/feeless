use crate::cli::StringOrStdin;
use crate::phrase::{Language, MnemonicType};
use clap::Clap;
use std::str::FromStr;

#[derive(Clap)]
pub struct PhraseOpts {
    #[clap(subcommand)]
    command: Command,
}

impl PhraseOpts {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::New(x) => {
                let phrase = crate::Phrase::random(x.words.0, x.language.language.to_owned());
                println!("{}", phrase);
            }
            Command::ToPrivate(x) => {
                let private = x.opts.to_private()?;
                println!("{}", private);
            }
            Command::ToPublic(x) => {
                let public = x.opts.to_private()?.to_public()?;
                println!("{}", public);
            }
            Command::ToAddress(x) => {
                let address = x.opts.to_private()?.to_public()?.to_address();
                println!("{}", address);
            }
        }
        Ok(())
    }
}

pub struct WrappedMnemonicType(pub(crate) MnemonicType);

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
    New(New),
    ToPrivate(Private),
    ToPublic(Public),
    ToAddress(Address),
}

// This is used with `#[clap(flatten)]` to prevent have duplicate code.
#[derive(Clap)]
pub struct LanguageOpt {
    /// Word list language: en, zh-hans, zh-hant, fr, it, ja, ko, es
    #[clap(short, long, default_value = "en")]
    pub(crate) language: Language,
}

/// Generate a random phrase. By default the word list is English with 24 words.
#[derive(Clap)]
pub struct New {
    /// Number of words. Possible values are: 12, 15, 18, 21, 24.
    #[clap(short, long, default_value = "24")]
    pub(crate) words: WrappedMnemonicType,

    #[clap(flatten)]
    pub(crate) language: LanguageOpt,
}

#[derive(Clap)]
pub struct FromPhraseOpts {
    // Keep this as String because we need `phrase_opts` to work out how to convert into a Phrase.
    words: StringOrStdin<String>,

    #[clap(flatten)]
    language: LanguageOpt,

    #[clap(short, long, default_value = "0")]
    account: u32,

    // I tried using default_value = "" but clap still complained about the field being required.
    #[clap(short, long)]
    passphrase: Option<String>,
}

impl FromPhraseOpts {
    pub fn to_private(&self) -> anyhow::Result<crate::Private> {
        let words = self.words.to_owned().resolve().unwrap();
        let phrase = crate::Phrase::from_words(self.language.language.to_owned(), words.as_str())?;
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

/// Convert a phrase to a public key.
#[derive(Clap)]
pub struct Public {
    #[clap(flatten)]
    opts: FromPhraseOpts,
}

/// Convert a phrase to an address.
#[derive(Clap)]
pub struct Address {
    #[clap(flatten)]
    opts: FromPhraseOpts,
}
