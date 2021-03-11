use crate::cli::OptionPipe;
use clap::Clap;

#[derive(Clap)]
pub struct Public {
    /// The public key in hex or `-` if reading from stdin.
    pub key: OptionPipe<crate::Public>,

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
