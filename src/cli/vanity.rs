use crate::vanity;
use crate::vanity::Secret;
use clap::Clap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration, Instant};

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

        let mut vanity = vanity::Vanity::new(secret_type, matches);
        if let Some(tasks) = opts.tasks {
            vanity.tasks(tasks);
        }
        if opts.include_digit {
            vanity.include_first_digit(true);
        }

        let (mut rx, attempts) = vanity.start().await?;
        let started = Instant::now();
        let mut last_log = Instant::now();
        let mut found = 0;
        loop {
            match timeout(Duration::from_secs(1), rx.recv()).await {
                Ok(Some(result)) => {
                    let s = match result.secret {
                        Secret::Phrase(p) => p.to_string(),
                        Secret::Seed(s) => s.to_string(),
                        Secret::Private(p) => p.to_string(),
                    };
                    println!("{},{}", result.address.to_string(), s);
                    last_log = log(started, last_log, attempts.clone()).await;
                    found += 1;
                    if let Some(limit) = opts.limit {
                        if limit == found {
                            break;
                        }
                    }
                }
                // Channel closed.
                Ok(None) => {
                    break;
                }
                // Timeout
                Err(_) => {
                    last_log = log(started, last_log, attempts.clone()).await;
                }
            }
        }
        Ok(())
    }
}

async fn log(started: Instant, last_log: Instant, attempts: Arc<RwLock<usize>>) -> Instant {
    let now = Instant::now();
    let since_last_log = now.duration_since(last_log);
    if since_last_log < Duration::from_secs(1) {
        last_log
    } else {
        let total_taken = Instant::now().duration_since(started);
        let attempts = *attempts.read().await;
        let rate = (attempts as f64) / total_taken.as_secs_f64();
        eprintln!("Attempted: {}, Rate: {:?} attempts/s", attempts, rate);
        now
    }
}

#[derive(Clap)]
enum VanitySecretType {
    /// Generate phrases to find addresses. WARNING: This method is slow currently.
    Phrase(PhraseOpts),
    /// Generate seeds to find addresses.
    Seed(SeedOpts),
    /// Generate private keys to find addresses.
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

    /// When matching, also match against the first digit (1 or 3) after `nano_`.
    #[clap(short, long)]
    include_digit: bool,

    /// Number of parallel tasks to use. Default: Your logical processors minus one, or at least 1.
    #[clap(short, long)]
    tasks: Option<usize>,

    /// Stop after finding this many matches.
    #[clap(short, long)]
    limit: Option<usize>,
}
