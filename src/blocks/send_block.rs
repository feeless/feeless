use crate::blocks::{hash_block, Block};
use crate::keys::public::{from_address, to_address};
use crate::raw::{deserialize_from_hex, serialize_to_hex};
use crate::{BlockHash, FullBlock, Public, Raw};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SendBlock {
    pub previous: BlockHash,
    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub destination: Public,
    #[serde(
        serialize_with = "serialize_to_hex",
        deserialize_with = "deserialize_from_hex"
    )]
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
