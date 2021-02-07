use crate::header::{BlockType, Header};
use crate::state::State;
use crate::wire::Wire;
use feeless::{expect_len, Address, Block, BlockHash, Public, Signature};
use std::convert::TryFrom;

pub struct Bytes<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    // TODO: Does this need to be safer?
    //       The offsets given are for already known sizes.
    pub fn slice(&mut self, size: usize) -> &[u8] {
        let bytes = &self.bytes[self.offset..self.offset + size];
        self.offset += size;
        bytes
    }

    pub fn u8(&mut self) -> u8 {
        let b = self.bytes[self.offset];
        self.offset += 1;
        b
    }
}

#[derive(Debug)]
pub struct ConfirmAck {
    account: Public,
    signature: Signature,
    sequence: u8,
    vote_by_hash: Vec<BlockHash>,
    block: Block,
}

impl ConfirmAck {}

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

        let account = Public::try_from(data.slice(Public::LEN))?;
        let signature = Signature::try_from(data.slice(Signature::LEN))?;
        let sequence = data.u8();
        let mut block_hashes = vec![];
        for _ in 0..header.ext().item_count() {
            block_hashes.push(BlockHash::try_from(data.slice(BlockHash::LEN))?);
        }
        // let block = Block;

        dbg!(account, signature, sequence, block_hashes);

        todo!()
    }

    fn len(header: Option<&Header>) -> usize {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        Public::LEN + Signature::LEN + 1 + BlockHash::LEN * header.ext().item_count()
        // TODO: Block
        // + Block::len(header.ext().block_type())
    }
}

#[derive(Debug)]
pub struct RootHashPair {
    pub hash: BlockHash,
    pub root: BlockHash,
}

impl RootHashPair {
    const LEN: usize = BlockHash::LEN * 2;
}

impl TryFrom<&[u8]> for RootHashPair {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Root hash pair")?;
        Ok(Self {
            hash: BlockHash::try_from(&value[0..BlockHash::LEN])?,
            root: BlockHash::try_from(&value[BlockHash::LEN..])?,
        })
    }
}
