use crate::node::wire::header::Header;
use crate::node::wire::Wire;
use crate::Raw;

struct WireRaw(Raw);

impl Wire for Raw {
    fn serialize(&self) -> Vec<u8> {
        // self.0
        todo!()
    }

    fn deserialize(_: Option<&Header>, _data: &[u8]) -> Result<Self, anyhow::Error>
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

    #[test]
    fn serialize() {}
}
