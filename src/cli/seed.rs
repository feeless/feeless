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
            Command::Private(o) => {
                let private = o.seed.to_owned().resolve()?.derive(o.index);
                println!("{}", private)
            }
            Command::Public(o) => {
                let public = o.seed.to_owned().resolve()?.derive(o.index).to_public()?;
                println!("{}", public)
            }
            Command::Address(o) => {
                let address = o
                    .seed
                    .to_owned()
                    .resolve()?
                    .derive(o.index)
                    .to_public()?
                    .to_address();
                println!("{}", address)
            }
        }
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    New,
    Private(Opts),
    Public(Opts),
    Address(Opts),
}

#[derive(Clap)]
pub struct Opts {
    seed: StringOrStdin<crate::Seed>,

    #[clap(short, long, default_value = "0")]
    index: u32,
}
