use crate::node::wire::header::Header;
use crate::node::wire::Wire;

#[derive(Debug)]
pub struct Empty;

impl Wire for Empty {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn deserialize(_: Option<&Header>, _data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(0)
    }
}
