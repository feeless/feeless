use crate::blocks::{hash_block, Block};
use crate::encoding::blake2b;
use crate::{blocks, Address, BlockHash, FullBlock, Public};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenBlock {
    pub source: Public,
    pub representative: Public,
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
