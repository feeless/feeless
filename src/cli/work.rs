use crate::blocks::BlockHash;
use crate::pow::{Subject, Work};
use crate::Difficulty;
use clap::Clap;
use std::str::FromStr;
use tracing::info;

#[derive(Clap)]
pub struct WorkOpts {
    /// The public key hash or block hash to be worked on in hex.
    hash: BlockHash,

    /// Use the base difficulty for a normal block.
    #[clap(short, long, group = "base")]
    normal: bool,

    /// Use the base difficulty for a receive block.
    #[clap(short, long, group = "base")]
    receive: bool,

    /// The base difficulty in hex.
    #[clap(short, long, group = "base")]
    difficulty: Option<Difficulty>,
}

impl WorkOpts {
    pub fn handle(&self) -> anyhow::Result<()> {
        // This is a bit hacky. We don't know if the user is giving a public key or a block hash.
        // It really doesn't matter which it is, pow doesn't care, so we just pick one.
        let subject = Subject::Hash(self.hash.to_owned());

        let difficulty = if let Some(d) = &self.difficulty {
            d.to_owned()
        } else if self.receive {
            Difficulty::receive()
        } else {
            Difficulty::normal()
        };
        info!("Finding work for {:?} at {:?}", &subject, &difficulty);
        let result = Work::generate(&subject, &difficulty)?;
        dbg!(result);
        Ok(())
    }
}
