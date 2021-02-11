use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::StateBlock;

#[derive(Debug)]
pub struct Publish(StateBlock);

impl Wire for Publish {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Self(StateBlock::deserialize(None, data)?))
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(StateBlock::LEN)
    }
}
