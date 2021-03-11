use crate::cli::StringOrStdin;
use clap::Clap;

#[derive(Clap)]
pub struct Public {
    pub key: StringOrStdin<crate::Public>,

    #[clap(subcommand)]
    pub to: PublicTo,
}

impl Public {
    pub fn handle(&self) -> anyhow::Result<()> {
        let public = self.key.to_owned().resolve()?;
        match self.to {
            PublicTo::Address => println!("{}", public.to_address().to_string()),
        };
        Ok(())
    }
}

#[derive(Clap)]
pub enum PublicTo {
    Address,
}
