use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct Public {
    #[clap(subcommand)]
    command: Command,
}

impl Public {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Address(a) => println!("{}", a.public.to_owned().resolve()?.to_address()),
        };
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    Address(Address),
}

#[derive(Clap)]
pub struct Address {
    public: StringOrStdin<crate::Public>,
}
