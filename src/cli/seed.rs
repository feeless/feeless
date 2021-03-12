use crate::cli::StringOrStdin;
use anyhow::anyhow;
use clap::Clap;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Clap)]
pub struct Seed {
    #[clap(subcommand)]
    command: Command,
}

impl Seed {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::New => println!("{}", crate::Seed::random()),
            Command::Private(p) => println!("{}", p.seed.to_owned().resolve()?.derive(p.index)),
        }
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    New,
    Private(Private),
}

#[derive(Clap)]
pub struct Private {
    seed: StringOrStdin<crate::Seed>,

    #[clap(short, long, default_value = "0")]
    index: u32,
}
