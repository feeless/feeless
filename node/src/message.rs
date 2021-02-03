use crate::cookie::Cookie;
use crate::state::State;
use crate::wire::Wire;
use feeless::Public;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct NodeIdHandshakeQuery(pub Cookie);

impl Wire for NodeIdHandshakeQuery {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: &State, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(NodeIdHandshakeQuery(Cookie::try_from(data)?))
    }

    fn len() -> usize {
        Cookie::LENGTH
    }
}

pub struct NodeIdHandshakeResponse {
    pub address: Public,
    pub signature: [u8; 64], // TODO: Signature
}

impl NodeIdHandshakeResponse {
    pub const LENGTH: usize = Cookie::LENGTH;
}
