use crate::blocks::BlockHash;
use crate::{Public, Signature, Work};
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Clap)]
pub struct ChangeBlock {
    #[clap(short, long)]
    previous: BlockHash,

    #[clap(short, long)]
    representative: Public,

    #[clap(short, long)]
    pub work: Option<Work>,

    #[clap(short = 'g', long)]
    pub signature: Option<Signature>,
}
