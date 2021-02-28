use crate::bytes::Bytes;
use crate::node::cookie::Cookie;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{Public, Signature};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Handshake {
    pub query: Option<HandshakeQuery>,
    pub response: Option<HandshakeResponse>,
}

impl Wire for Handshake {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        debug_assert!(header.is_some());
        let header = header.unwrap();
        let mut bytes = Bytes::new(data);
        let mut s = Self {
            query: None,
            response: None,
        };

        if header.ext().is_query() {
            s.query = Some(HandshakeQuery::deserialize(
                Some(header),
                bytes.slice(HandshakeQuery::LEN)?,
            )?);
        }
        if header.ext().is_response() {
            s.response = Some(HandshakeResponse::deserialize(
                Some(header),
                bytes.slice(HandshakeResponse::LEN)?,
            )?);
        }
        Ok(s)
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error> {
        debug_assert!(header.is_some());
        let header = header.unwrap();
        let mut size = 0;
        if header.ext().is_query() {
            size += HandshakeQuery::LEN
        }
        if header.ext().is_response() {
            size += HandshakeResponse::LEN
        };
        Ok(size)
    }
}

#[derive(Debug)]
pub struct HandshakeQuery(pub Cookie);

impl<'a> HandshakeQuery {
    const LEN: usize = Cookie::LEN;

    pub fn new(cookie: Cookie) -> Self {
        Self(cookie)
    }

    pub fn cookie(&self) -> &Cookie {
        &self.0
    }
}

impl Wire for HandshakeQuery {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let cookie = Cookie::deserialize(header, data)?;
        Ok(HandshakeQuery(cookie))
    }

    fn len(_: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Self::LEN)
    }
}

#[derive(Debug)]
pub struct HandshakeResponse {
    pub public: Public,
    pub signature: Signature,
}

impl HandshakeResponse {
    pub const LEN: usize = Public::LEN + Signature::LEN;

    pub fn new(public: Public, signature: Signature) -> Self {
        Self { public, signature }
    }
}

impl Wire for HandshakeResponse {
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
