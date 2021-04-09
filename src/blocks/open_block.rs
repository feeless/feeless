use crate::blocks::BlockHash;
use crate::keys::public::{from_address, to_address};
use crate::{Public, Signature, Work};
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Clap)]
pub struct OpenBlock {
    /// BlockHash of the Open block sending the funds to this account.
    #[clap(short, long)]
    pub source: BlockHash,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    #[clap(short, long)]
    pub representative: Public,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    #[clap(short, long)]
    pub account: Public,

    #[clap(short, long)]
    pub work: Option<Work>,

    #[clap(short = 'g', long)]
    pub signature: Option<Signature>,
}

impl OpenBlock {
    pub fn new(source: BlockHash, representative: Public, account: Public) -> Self {
        Self {
            source,
            representative,
            account,
            work: None,
            signature: None,
        }
    }
}
