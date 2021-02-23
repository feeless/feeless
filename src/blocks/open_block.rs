use crate::blocks::{hash_block, Block};

use crate::keys::public::{from_address, to_address};
use crate::{BlockHash, Public, Signature, Work};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenBlock {
    /// BlockHash of the Open block sending the funds to this account.
    pub source: BlockHash,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub representative: Public,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub account: Public,

    pub work: Option<Work>,
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

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        hash_block(&[
            self.source.as_bytes(),
            self.representative.as_bytes(),
            self.account.as_bytes(),
        ])
    }
}
