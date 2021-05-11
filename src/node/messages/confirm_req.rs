use crate::blocks::{BlockHash, BlockHolder, BlockType};
use crate::bytes::Bytes;
use crate::encoding::expect_len;
use crate::node::header::Header;
use crate::node::wire::Wire;
use anyhow::Context;
use std::convert::TryFrom;
use tracing::info;

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
    BlockSelector(BlockHolder),
}

impl ConfirmReq {
    pub const CONFIRM_REQ_BY_HASH_LEN: usize = BlockHash::LEN * 2;
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

        let mut bytes = Bytes::new(data);

        if header.ext().block_type()? == BlockType::NotABlock {
            let count = header.ext().item_count() as usize;
            let expected_capacity = RootHashPair::LEN * count;
            expect_len(
                data.len(),
                expected_capacity,
                "HandleConfirmReq root hash pairs",
            )?;

            let mut pairs = Vec::with_capacity(expected_capacity);
            for _ in 0..count {
                let value = bytes
                    .slice(RootHashPair::LEN)
                    .context("Confirm req slicing root hash pair")?;
                let pair = RootHashPair::try_from(value).context("Confirm req try from bytes")?;
                pairs.push(pair);
            }
            Ok(Self::ConfirmReqByHash(pairs))
        } else {
            info!("Block type {:?}", header.ext().block_type());

            Ok(Self::BlockSelector(BlockHolder::deserialize(
                Some(header),
                data,
            )?))

            // todo!("handle state block")
            //
            // let value = bytes
            //     .slice(FullBlock::LEN)
            //     .context("Confirm req slice state block")?;
            // Ok(Self::BlockSelector(
            //     FullBlock::deserialize(Some(header), value)
            //         .context("Confirm req block selector state block deserialize")?,
            // ))
        }
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize> {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type()? == BlockType::NotABlock {
            Ok(Self::CONFIRM_REQ_BY_HASH_LEN * header.ext().item_count())
        } else {
            BlockHolder::len(Some(header))
        }
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
