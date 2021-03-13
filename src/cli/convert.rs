use crate::encoding::FromHex;
use crate::Public;
use anyhow::Context;
use clap::Clap;

#[derive(Clap)]
pub struct ConvertFrom {
    #[clap(subcommand)]
    pub command: ConvertFromCommand,
}

/// Conversions between types, e.g. public key to nano addkkress.
#[derive(Clap)]
pub enum ConvertFromCommand {
    Public(ConvertFromPublic),
}

impl ConvertFromCommand {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self {
            ConvertFromCommand::Public(public) => {
                let address = Public::from_hex(&public.public_key)?.to_address();
                println!("{}", address);
            }
        }
        Ok(())
    }
}

/// Convert from a public key in hex.
#[derive(Clap)]
pub struct ConvertFromPublic {
    public_key: String,
}
