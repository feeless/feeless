#[cfg(feature = "node")]
use crate::node::header::Header;
#[cfg(feature = "node")]
use crate::node::wire::Wire;

use crate::bytes::Bytes;
use crate::encoding::blake2b;
use crate::{expect_len, Address, BlockHash, Public, Raw, Signature, Work};
use anyhow::anyhow;
use change_block::ChangeBlock;
use core::convert::TryFrom;
use open_block::OpenBlock;
use receive_block::ReceiveBlock;
use send_block::SendBlock;
use state_block::{Link, StateBlock};
use std::hash::Hash;
use tracing::warn;
pub mod block_hash;
pub mod change_block;
pub mod open_block;
pub mod receive_block;
pub mod send_block;
pub mod state_block;

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

#[derive(Debug)]
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
#[derive(Debug)]
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

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        match &self.block {
            // Block::Send(x) => x.hash(),
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
}

pub fn hash_block(parts: &[&[u8]]) -> anyhow::Result<BlockHash> {
    let mut v = Vec::new(); // TODO: with_capacity
    for b in parts {
        v.extend_from_slice(b);
    }
    BlockHash::try_from(blake2b(BlockHash::LEN, &v).as_ref())
}
