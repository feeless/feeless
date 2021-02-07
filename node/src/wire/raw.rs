use crate::header::Header;
use crate::wire::Wire;
use feeless::Raw;

struct WireRaw(Raw);

impl Wire for Raw {
    fn serialize(&self) -> Vec<u8> {
        // self.0
        todo!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn len(_: Option<&Header>) -> Result<usize, anyhow::Error> {
        Ok(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {}
}
