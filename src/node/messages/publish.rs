use crate::blocks::BlockHolder;
use crate::node::header::Header;
use crate::node::wire::Wire;

#[derive(Debug)]
pub struct Publish(pub(crate) BlockHolder);

impl Wire for Publish {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Publish(BlockHolder::deserialize(header, data)?))
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error> {
        BlockHolder::len(header)
    }
}
