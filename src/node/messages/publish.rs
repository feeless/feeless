use crate::blocks::FullBlock;
use crate::node::header::Header;
use crate::node::wire::Wire;

#[derive(Debug)]
pub struct Publish(FullBlock);

impl Wire for Publish {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        todo!("handle full block")
        // Ok(Self(FullBlock::deserialize(None, data)?))
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        todo!("handle full block")
        // Ok(FullBlock::LEN)
    }
}
