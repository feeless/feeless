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
            Command::Random(r) => r.handle(),
        }
    }
}

#[derive(Clap)]
pub enum Command {
    Random(Random),
}

#[derive(Clap)]
pub struct Random {
    #[clap(short, long, default_value = "24")]
    words: WrappedMnemonicType,

    #[clap(short, long, default_value = "en")]
    language: WrappedLanguage,
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

impl Random {
    pub fn handle(&self) -> anyhow::Result<()> {
        println!("{}", crate::Phrase::random(self.words.0, self.language.0));
        Ok(())
    }
}
