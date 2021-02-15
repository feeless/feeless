use core::convert::TryFrom;
use std::hash::Hash;

use anyhow::anyhow;
use tracing::warn;

use change_block::ChangeBlock;
use open_block::OpenBlock;
use receive_block::ReceiveBlock;
use send_block::SendBlock;
use state_block::{Link, StateBlock};

use crate::bytes::Bytes;
use crate::encoding::blake2b;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{expect_len, Address, BlockHash, Public, Raw, Signature, Work};

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

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        match &self.block {
            // Block::Send(x) => x.hash(),
            // Block::Receive(x) => x.hash(),
            Block::Open(x) => x.hash(),
            // Block::Change(x) => x.hash(),
            // Block::State(x) => x.hash(),
            _ => todo!(),
        }
    }

    pub fn set_work(&mut self, work: Work) -> anyhow::Result<()> {
        self.work = Some(work);
        Ok(())
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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::state_block::{Link, StateBlock};
    use super::{Address, BlockHash, Raw, Signature, Work};
    use super::{Block, FullBlock};

    #[test]
    fn hash() {
        let account =
            Address::from_str("nano_34prihdxwz3u4ps8qjnn14p7ujyewkoxkwyxm3u665it8rg5rdqw84qrypzk")
                .unwrap()
                .to_public();
        let parent =
            BlockHash::from_hex("7837C80964CAD551DEABE162C7FC4BB58688A0C6EB6D9907C0D2A7C74A33C7EB")
                .unwrap();
        let representative = account.clone();
        let balance = Raw::from_raw(2711469892748129430069222848295u128);
        let link = Link::PairingSendBlockHash(
            BlockHash::from_hex("0399B19B022D260F3DDFBA26D0306D423F1890D3AE06136FAB16802D1F2B87A7")
                .unwrap(),
        );
        // Signature and work aren't hashed, but left them as the real data anyway.
        let signature = Signature::from_hex("BCF9F123138355AE9E741912D319FF48E5FCCA39D9E5DD74411D32C69B1C7501A0BF001C45D4F68CB561B902A42711E6166B9018E76C50CC868EF2E32B78F200").unwrap();
        let work = Work::from_hex("d4757052401b9e08").unwrap();

        todo!();
        // let block = StateBlock::new(account, parent, representative, balance, link).to_full();
        // block.set_signature(signature);
        // block.set_work(work);
        //
        // assert_eq!(
        //     block.hash().unwrap(),
        //     BlockHash::from_hex("6F050D3D0B19C2C206046AAE2D46661B57E1B7D890DE8398D203A025E29A4AD9")
        //         .unwrap()
        // )
    }
}
