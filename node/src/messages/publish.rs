use crate::header::Header;
use crate::wire::Wire;
use feeless::StateBlock;
use tracing::warn;

#[derive(Debug)]
struct Publish(StateBlock);

impl Wire for Publish {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        warn!("TODO deserialize publish");
        Ok(Self {})
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(StateBlock::LEN)
    }
}
