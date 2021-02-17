use crate::blocks::{hash_block, Block};
use crate::{BlockHash, FullBlock, Public, Raw};

#[derive(Debug, Clone)]
pub struct SendBlock {
    pub previous: BlockHash,
    pub destination: Public,
    pub balance: Raw,
}

impl SendBlock {
    pub fn new(previous: BlockHash, destination: Public, balance: Raw) -> Self {
        Self {
            previous,
            destination,
            balance,
        }
    }

    pub fn into_full_block(self) -> FullBlock {
        FullBlock::new(Block::Send(self))
    }

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        hash_block(&[
            self.previous.as_bytes(),
            self.destination.as_bytes(),
            self.balance.to_vec().as_slice(),
        ])
    }
}
