use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct Private {
    #[clap(subcommand)]
    command: Command,
}

impl Private {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::ToPublic(a) => {
                let public = a.private.to_owned().resolve()?.to_public()?;
                println!("{}", public);
            }
            Command::ToAddress(a) => {
                let address = a.private.to_owned().resolve()?.to_public()?.to_address();
                println!("{}", address);
            }
        };
        Ok(())
    }
}

#[derive(Clap)]
pub enum Command {
    ToPublic(Public),
    ToAddress(Address),
}

#[derive(Clap)]
pub struct Public {
    private: StringOrStdin<crate::Private>,
}

#[derive(Clap)]
pub struct Address {
    private: StringOrStdin<crate::Private>,
}
