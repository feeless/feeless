use crate::bytes::Bytes;
use crate::node::peer_info::PeerInfo;
use crate::transport::header::Header;
use crate::transport::wire::Wire;

#[derive(Debug)]
pub struct Keepalive(Vec<PeerInfo>);

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
            let slice = bytes.slice(PeerInfo::LEN)?;
            if slice == [0u8; PeerInfo::LEN] {
                continue;
            }
            let peer = PeerInfo::deserialize(header, slice)?;
            s.0.push(peer);
        }
        Ok(s)
    }

    fn len(_: Option<&Header>) -> anyhow::Result<usize> {
        Ok(PeerInfo::LEN * Keepalive::PEERS)
    }
}
