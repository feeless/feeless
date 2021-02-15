use std::convert::TryFrom;

use crate::blocks::Block;
use crate::encoding::blake2b;
use crate::{blocks, Address, BlockHash, FullBlock, Public};

#[derive(Debug, Clone)]
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

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        blocks::hash_block(&[
            self.source.as_bytes(),
            self.representative.as_bytes(),
            self.account.as_bytes(),
        ])
    }

    pub fn into_full_block(self) -> FullBlock {
        FullBlock::new(Block::Open(self))
    }
}
