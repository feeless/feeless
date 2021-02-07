use crate::header::{BlockType, Header};
use crate::state::SledState;
use crate::wire::Wire;
use feeless::{expect_len, BlockHash};
use std::convert::TryFrom;

/// Requests confirmation of the given block or list of root/hash pairs.
//
// seq:
//  - id: reqbyhash
//    if: _root.header.block_type == enum_blocktype::not_a_block
//    type: confirm_request_by_hash
//  - id: block
//    if: _root.header.block_type != enum_blocktype::not_a_block
//    type: block_selector(_root.header.block_type_int)
#[derive(Debug)]
pub enum ConfirmReq {
    ConfirmReqByHash(Vec<RootHashPair>),
    BlockSelector,
}

impl ConfirmReq {
    pub const LEN: usize = BlockHash::LEN * 2;
}

impl Wire for ConfirmReq {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type()? == BlockType::NotABlock {
            let count = header.ext().item_count() as usize;
            let expected_capacity = RootHashPair::LEN * count;
            expect_len(
                data.len(),
                expected_capacity,
                "HandleConfirmReq root hash pairs",
            )?;

            let mut pairs = Vec::with_capacity(expected_capacity);
            for idx in 0..count {
                let offset = idx * RootHashPair::LEN;
                let pair = RootHashPair::try_from(&data[offset..offset + RootHashPair::LEN])?;
                pairs.push(pair);
            }
            Ok(Self::ConfirmReqByHash(pairs))
        } else {
            todo!("unhandled HandleConfirmReq for BlockSelector")
        }
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize> {
        Ok(BlockHash::LEN * 2)
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
