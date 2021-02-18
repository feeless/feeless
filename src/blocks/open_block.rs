use crate::blocks::{hash_block, Block};
use crate::encoding::blake2b;
use crate::keys::public::{from_address, to_address};
use crate::{blocks, Address, BlockHash, FullBlock, Public};
use serde::{Deserialize, Serialize, Serializer};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenBlock {
    pub source: Public,
    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub representative: Public,
    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub account: Public,
}

impl OpenBlock {
    pub fn new(source: Public, representative: Public, account: Public) -> Self {
        Self {
            source,
            representative,
            account,
        }
    }

    pub fn into_full_block(self) -> FullBlock {
        FullBlock::new(Block::Open(self))
    }

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        hash_block(&[
            self.source.as_bytes(),
            self.representative.as_bytes(),
            self.account.as_bytes(),
        ])
    }
}
