use crate::header::Header;
use crate::wire::Wire;

#[derive(Debug)]
pub struct TelemetryReq;

impl Wire for TelemetryReq {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(0)
    }
}
