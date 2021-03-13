use crate::cli::StringOrStdin;

use clap::Clap;



#[derive(Clap)]
pub struct Seed {
    #[clap(subcommand)]
    command: Command,
}

impl Seed {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::New => println!("{}", crate::Seed::random()),
            Command::ToPrivate(o) => {
                let private = o.seed.to_owned().resolve()?.derive(o.index);
                println!("{}", private)
            }
            Command::ToPublic(o) => {
                let public = o.seed.to_owned().resolve()?.derive(o.index).to_public()?;
                println!("{}", public)
            }
            Command::ToAddress(o) => {
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
    ToPrivate(Opts),
    ToPublic(Opts),
    ToAddress(Opts),
}

#[derive(Clap)]
pub struct Opts {
    seed: StringOrStdin<crate::Seed>,

    #[clap(short, long, default_value = "0")]
    index: u32,
}
