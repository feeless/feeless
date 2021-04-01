use crate::keys::armor::Armor;
use crate::{Address, Public, Signature};
use anyhow::anyhow;
use clap::Clap;
use std::io;
use std::io::Read;
use std::str::FromStr;

#[derive(Clap)]
pub struct VerifyOpts {
    #[clap(short, long, group = "pub")]
    address: Option<Address>,

    #[clap(short, long, group = "pub")]
    public: Option<Public>,

    #[clap(short, long)]
    signature: Option<Signature>,

    #[clap(short, long)]
    message: Option<String>,

    #[clap(long)]
    armor: bool,
}

impl VerifyOpts {
    pub(crate) fn handle(&self) -> anyhow::Result<()> {
        if self.armor {
            self.handle_armor()?;
        } else {
            self.handle_args()?;
        }

        println!("OK");
        Ok(())
    }

    fn handle_armor(&self) -> anyhow::Result<()> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let armor = Armor::from_str(&buffer)?;
        armor.verify()?;
        Ok(())
    }

    fn handle_args(&self) -> anyhow::Result<()> {
        let message = if let Some(message) = &self.message {
            message
        } else {
            return Err(anyhow!("Please specify a message."));
        };

        let signature = if let Some(signature) = &self.signature {
            signature
        } else {
            return Err(anyhow!("Please specify a signature."));
        };

        let public = if let Some(address) = &self.address {
            address.to_public()
        } else if let Some(public) = &self.public {
            public.to_owned()
        } else {
            return Err(anyhow!("Please specify an address or public key."));
        };

        public.verify(message.as_bytes(), signature)?;
        Ok(())
    }
}
