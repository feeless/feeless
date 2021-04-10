use crate::blocks::BlockHash;
use crate::{Public, Signature, Work};
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Clap)]
pub struct ReceiveBlock {
    #[clap(short, long)]
    previous: BlockHash,

    #[clap(short, long)]
    source: Public,

    #[clap(short, long)]
    pub work: Option<Work>,

    #[clap(short = 'g', long)]
    pub signature: Option<Signature>,
}
