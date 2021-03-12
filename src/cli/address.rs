use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct Address {
    address: StringOrStdin<crate::Address>,

    #[clap(subcommand)]
    command: Command,
}

impl Address {
    pub fn handle(&self) -> anyhow::Result<()> {
        let address = self.address.to_owned().resolve()?;
        match &self.command {
            Command::Validate => {}
            Command::Public => {
                println!("{}", address);
            }
        }
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    Validate,
    Public,
}
