use std::convert::TryFrom;

use crate::blocks::{Block, BlockType};
use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{BlockHash, Public, Signature};

#[derive(Debug)]
pub struct ConfirmAck {
    // TODO: Make a signed public that's shared with handshake.
    account: Public,
    signature: Signature,

    sequence: [u8; ConfirmAck::SEQUENCE_LEN],
    confirm: Confirm,
}

#[derive(Debug)]
pub enum Confirm {
    VoteByHash(Vec<BlockHash>),
    Block(Block), // TODO: this variant is 704 bytes, use Box<>?
}

impl ConfirmAck {
    const SEQUENCE_LEN: usize = 8;
    const VOTE_COMMON_LEN: usize = Public::LEN + Signature::LEN + Self::SEQUENCE_LEN;

    pub fn new(account: Public, signature: Signature, sequence: &[u8], confirm: Confirm) -> Self {
        let mut s = Self {
            account,
            signature,
            sequence: [0u8; ConfirmAck::SEQUENCE_LEN],
            confirm,
        };
        s.sequence.copy_from_slice(sequence);
        s
    }
}

impl Wire for ConfirmAck {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        let mut data = Bytes::new(data);
        let account = Public::try_from(data.slice(Public::LEN)?)?;
        let signature = Signature::try_from(data.slice(Signature::LEN)?)?;
        // to_vec here to stop a borrow problem
        let sequence = data.slice(ConfirmAck::SEQUENCE_LEN)?.to_vec();
        let confirm = if header.ext().block_type()? == BlockType::NotABlock {
            let mut block_hashes = vec![];
            for _ in 0..header.ext().item_count() {
                block_hashes.push(BlockHash::try_from(data.slice(BlockHash::LEN)?)?);
            }
            Confirm::VoteByHash(block_hashes)
        } else {
            // let block = Block;
            dbg!("block!!!!!!!", header.ext().block_type().unwrap());
            // dbg!("{:X}", data.slice(FullBlock::LEN)?);
            todo!()
        };

        Ok(Self::new(account, signature, &sequence, confirm))
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize> {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type()? == BlockType::NotABlock {
            Ok(Self::VOTE_COMMON_LEN + header.ext().item_count() * BlockHash::LEN)
        } else {
            dbg!(header);
            todo!("got a block in confirm ack");
            // Ok(Self::VOTE_COMMON_LEN + FullBlock::LEN)
        }
    }
}
