#[cfg(feature = "node")]
use crate::node::header::Header;
#[cfg(feature = "node")]
use crate::node::wire::Wire;

use crate::bytes::Bytes;
use crate::encoding::blake2b;
use crate::{expect_len, Address, Private, Public, Raw, Signature, Work};
use anyhow::anyhow;
pub use block_hash::BlockHash;
pub use change_block::ChangeBlock;
use core::convert::TryFrom;
pub use open_block::OpenBlock;
pub use receive_block::ReceiveBlock;
pub use send_block::SendBlock;
use serde::{Deserialize, Serialize};
pub use state_block::{Link, StateBlock};
use std::hash::Hash;
use tracing::warn;

mod block_hash;
mod change_block;
mod open_block;
mod receive_block;
mod send_block;
mod state_block;

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Invalid = 0,
    NotABlock = 1,
    Send = 2,
    Receive = 3,
    Open = 4,
    Change = 5,
    State = 6,
}

impl TryFrom<u8> for BlockType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use BlockType::*;
        Ok(match value {
            0 => Invalid,
            1 => NotABlock,
            2 => Send,
            3 => Receive,
            4 => Open,
            5 => Change,
            6 => State,
            _ => return Err(anyhow!("Invalid block type: {}", value)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Block {
    Send(SendBlock),
    Receive(ReceiveBlock),
    Open(OpenBlock),
    Change(ChangeBlock),
    State(StateBlock),
}

/// A FullBlock contains all block information needed for the network.
///
/// It includes work and signature, as well as the block specific information based on its type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FullBlock {
    block: Block,
    work: Option<Work>,
    signature: Option<Signature>,
}

impl FullBlock {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            work: None,
            signature: None,
        }
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    // TODO: Can this ever fail?
    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        match &self.block {
            Block::Send(x) => x.hash(),
            // Block::Receive(x) => x.hash(),
            Block::Open(x) => x.hash(),
            // Block::Change(x) => x.hash(),
            Block::State(x) => x.hash(),
            _ => todo!(),
        }
    }

    pub fn work(&self) -> Option<&Work> {
        self.work.as_ref()
    }

    pub fn set_work(&mut self, work: Work) -> anyhow::Result<()> {
        self.work = Some(work);
        Ok(())
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn set_signature(&mut self, signature: Signature) -> anyhow::Result<()> {
        self.signature = Some(signature);
        Ok(())
    }

    pub fn sign(&mut self, private: Private) -> anyhow::Result<()> {
        let hash = self.hash()?;
        let signature = private.sign(hash.as_bytes())?;
        self.set_signature(signature)
    }

    /// If it's an open block, return it.
    pub fn open_block(&self) -> anyhow::Result<&OpenBlock> {
        if let Block::Open(o) = &self.block() {
            Ok(o)
        } else {
            Err(anyhow!("Not an open block"))
        }
    }

    /// If it's a send block, return it.
    pub fn send_block(&self) -> anyhow::Result<&SendBlock> {
        if let Block::Send(o) = &self.block() {
            Ok(o)
        } else {
            Err(anyhow!("Not an open block"))
        }
    }

    pub fn balance(&self) -> Option<&Raw> {
        match &self.block {
            Block::Send(b) => Some(&b.balance),
            _ => todo!(),
        }
    }

    pub fn previous(&self) -> Option<BlockHash> {
        match &self.block {
            Block::Open(b) => None,
            Block::Send(b) => Some(b.previous.to_owned()),
            _ => todo!(),
        }
    }
}

pub fn hash_block(parts: &[&[u8]]) -> anyhow::Result<BlockHash> {
    let mut v = Vec::new(); // TODO: with_capacity
    for b in parts {
        v.extend_from_slice(b);
    }
    BlockHash::try_from(blake2b(BlockHash::LEN, &v).as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::network::Network;

    #[test]
    fn json() {
        let genesis = r#"
        {
            "amount": "340282366920938463463374607431768211455",
            "balance": "340282366920938463463374607431768211455",
            "block_account": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
            "confirmed": "true",
            "contents": {
                "account": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
                "representative": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
                "signature": "9F0C933C8ADE004D808EA1985FA746A7E95BA2A38F867640F53EC8F180BDFE9E2C1268DEAD7C2664F356E37ABA362BC58E46DBA03E523A7B5A19E4B6EB12BB02",
                "source": "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
                "type": "open",
                "work": "62f05417dd3fb691"
            },
            "height": "1",
            "local_timestamp": "0"
        }
        "#;

        let block: FullBlock = serde_json::from_str(genesis).unwrap();
        assert_eq!(block, Network::Live.genesis_block());
    }
}
