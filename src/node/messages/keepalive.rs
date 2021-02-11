use crate::bytes::Bytes;
use crate::node::wire::header::Header;
use crate::node::wire::peer::Peer;
use crate::node::wire::Wire;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Keepalive(Vec<Peer>);

impl Keepalive {
    pub const PEERS: usize = 8;
}

impl Wire for Keepalive {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut s = Self(vec![]);
        let mut bytes = Bytes::new(data);
        for _ in 0..Keepalive::PEERS {
            let peer = Peer::deserialize(header, bytes.slice(Peer::LEN)?)?;
            s.0.push(peer);
        }
        Ok(s)
    }

    fn len(_: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Peer::LEN * Keepalive::PEERS)
    }
}
