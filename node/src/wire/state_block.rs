use crate::bytes::Bytes;
use crate::header::{BlockType, Header};
use crate::wire::Wire;
use feeless::{BlockHash, Public, Raw, Signature, StateBlock, Work};
use std::convert::TryFrom;

/// A wrapper around StateBlock with serialization.
pub struct WireStateBlock(StateBlock);

impl Wire for WireStateBlock {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        warn!("TODO Wire::deserialize");
        let mut data = Bytes::new(data);
        Self(StateBlock::new(
            Public::try_from(data.slice(Public::LEN)?),
            BlockHash::try_from(data.slice(BlockHash::LEN)?),
            Public::try_from(data.slice(Public::LEN)?),
            Raw::zero(), // TODO
            Public::try_from(data.slice(Public::LEN)?),
            Signature::try_from(data.slice(Public::LEN)?),
            Work::try_from(data.slice(Work::LEN)?),
        ))
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error> {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type() != BlockType::State {
            return Err(anyhow!(
                "Unexpected block type: {:?}",
                header.ext().block_type()
            ));
        }

        Ok(StateBlock::LEN)
    }
}
