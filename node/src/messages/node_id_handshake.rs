use crate::channel::Channel;
use crate::cookie::Cookie;
use crate::header::Header;
use crate::state::SledState;
use crate::wire::Wire;
use feeless::{Public, Signature};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct NodeIdHandshakeQuery(pub Cookie);

impl<'a> NodeIdHandshakeQuery {
    const LEN: usize = Cookie::LEN;

    pub fn new(cookie: Cookie) -> Self {
        Self(cookie)
    }

    pub fn cookie(&self) -> &Cookie {
        &self.0
    }
}

impl Wire for NodeIdHandshakeQuery {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let cookie = Cookie::deserialize(header, data)?;
        Ok(NodeIdHandshakeQuery(cookie))
    }

    fn len(_: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Self::LEN)
    }
}

#[derive(Debug)]
pub struct NodeIdHandshakeResponse {
    pub public: Public,
    pub signature: Signature,
}

impl NodeIdHandshakeResponse {
    pub const LEN: usize = Public::LEN + Signature::LEN;

    pub fn new(public: Public, signature: Signature) -> Self {
        Self { public, signature }
    }
}

impl Wire for NodeIdHandshakeResponse {
    fn serialize(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(Self::LEN);
        v.extend_from_slice(&self.public.as_bytes());
        v.extend_from_slice(&self.signature.as_bytes());
        v
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            public: Public::try_from(&data[0..Public::LEN])?,
            signature: Signature::try_from(&data[Public::LEN..])?,
        })
    }

    fn len(_: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Self::LEN)
    }
}

impl Channel {}
