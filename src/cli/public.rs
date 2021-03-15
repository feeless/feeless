use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct PublicOpts {
    #[clap(subcommand)]
    command: Command,
}

impl PublicOpts {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::ToAddress(a) => println!("{}", a.public.to_owned().resolve()?.to_address()),
        };
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    ToAddress(Address),
}

#[derive(Clap)]
pub struct Address {
    public: StringOrStdin<crate::Public>,
}
