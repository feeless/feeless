use crate::cli::Command::Vanity;
use crate::{vanity, Phrase};
use clap::Clap;
use std::str::FromStr;

#[derive(Clap)]
pub struct VanityOpts {
    #[clap(subcommand)]
    secret_type: VanitySecretType,
}

impl VanityOpts {
    pub async fn handle(&self) -> anyhow::Result<()> {
        let (secret_type, opts) = match &self.secret_type {
            VanitySecretType::Phrase(phrase) => {
                let secret_type = vanity::SecretType::Phrase {
                    language: phrase.phrase_opts.language.language.to_owned(),
                    words: phrase.phrase_opts.words.0,
                };
                (secret_type, &phrase.common_opts)
            }
            VanitySecretType::Seed(seed) => (vanity::SecretType::Seed, &seed.common_opts),
            VanitySecretType::Private(private) => {
                (vanity::SecretType::Private, &private.common_opts)
            }
        };

        let matches = if opts.start {
            vanity::Match::start(&opts.matching)
        } else if opts.end {
            vanity::Match::end(&opts.matching)
        } else if opts.regex {
            vanity::Match::regex(&opts.matching)?
        } else {
            vanity::Match::start_or_end(&opts.matching)
        };

        let vanity = vanity::Vanity::new(secret_type, matches);
        let mut rx = vanity.start().await?;
        while let Some(result) = rx.recv().await {
            dbg!(result);
        }
        Ok(())
    }
}

#[derive(Clap)]
enum VanitySecretType {
    Phrase(PhraseOpts),
    Seed(SeedOpts),
    Private(PrivateOpts),
}

#[derive(Clap)]
struct PhraseOpts {
    #[clap(flatten)]
    pub phrase_opts: super::phrase::New,

    #[clap(flatten)]
    pub common_opts: CommonOpts,
}

#[derive(Clap)]
struct SeedOpts {
    #[clap(flatten)]
    pub common_opts: CommonOpts,
}

#[derive(Clap)]
struct PrivateOpts {
    #[clap(flatten)]
    pub common_opts: CommonOpts,
}

#[derive(Clap)]
struct CommonOpts {
    /// Match on this string. By default will match the start and end.
    matching: String,

    /// Match on start only. Default is start and end.
    #[clap(short, long, group = "match")]
    start: bool,

    /// Match on end only. Default is start and end.
    #[clap(short, long, group = "match")]
    end: bool,

    /// Match on a regular expression instead.
    #[clap(short, long, group = "match")]
    regex: bool,

    #[clap(short, long, group = "output")]
    json: bool,

    #[clap(short, long, group = "output")]
    csv: bool,

    /// Number of parallel tasks to use. Default: Your logical processors minus one, or at least 1.
    #[clap(short, long)]
    tasks: Option<usize>,

    /// Stop after finding this many matches.
    #[clap(short, long, default_value = "1")]
    limit: usize,

    /// Quit after this many attempts
    #[clap(short, long)]
    quit: Option<usize>,
}

// fn parse_output_flag(s: &str) -> {
// }

// #[derive(Clap, Debug)]
// struct OutputFlag {
//     a: String,
// }
//
// impl FromStr for OutputFlag {
//     type Err = ();
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         unimplemented!()
//     }
// }

// #[derive(Clap, Debug)]
// enum OutputOpts {
//     How,
//     Work(String),
// }
//
// impl FromStr for OutputOpts {
//     type Err = anyhow::Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         unimplemented!()
//     }
// }
