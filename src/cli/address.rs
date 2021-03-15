use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct AddressOpts {
    #[clap(subcommand)]
    command: Command,
}

impl AddressOpts {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Public(p) => {
                println!("{}", p.address.to_owned().resolve()?.to_public());
            }
        }
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    Public(Public),
}

#[derive(Clap)]
pub struct Public {
    address: StringOrStdin<crate::Address>,
}
