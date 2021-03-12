use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct Public {
    key: StringOrStdin<crate::Public>,

    #[clap(subcommand)]
    command: Command,
}

impl Public {
    pub fn handle(&self) -> anyhow::Result<()> {
        let public = self.key.to_owned().resolve()?;
        match &self.command {
            Command::Address => println!("{}", public.to_address().to_string()),
        };
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    Address,
}
