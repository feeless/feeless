use crate::encoding::FromHex;
use crate::Public;
use anyhow::Context;
use clap::Clap;

// https://github.com/clap-rs/clap/issues/2005
// This shim struct required until the issue is fixed.
// It just temporarily adds another level to Opts.
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
                let public = Public::from_hex(&public.public_key).context(
                    "A valid public key is required, \
                    e.g. 0E90A70364120708F7CE4D527E66A0FCB9CB90E81054C4ED329C58EFA469F6F7",
                )?;
                println!("{}", public.to_address().to_string());
                Ok(())
            }
        }
    }
}

/// Convert from a public key in hex.
#[derive(Clap)]
pub struct ConvertFromPublic {
    public_key: String,
}
