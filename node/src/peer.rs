use crate::header::Header;
use crate::state::State;
use crate::wire::Wire;
use feeless::expect_len;
use std::convert::TryFrom;
use std::net::{Ipv6Addr, SocketAddrV6};
use std::str::FromStr;

#[derive(Debug)]
pub struct Peer(SocketAddrV6);

impl Peer {
    pub const LEN: usize = 18;
    pub const ADDR_LEN: usize = 16;

    pub fn socket_addr_v6(&self) -> SocketAddrV6 {
        self.0
    }
}

impl TryFrom<&str> for Peer {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Peer(SocketAddrV6::from_str(value)?))
    }
}

impl TryFrom<&[u8]> for Peer {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Peer")?;

        let mut addr: [u8; Self::ADDR_LEN] = [0u8; Self::ADDR_LEN];
        addr.copy_from_slice(&value[0..Self::ADDR_LEN]);
        let port: u16 = value[Self::ADDR_LEN] as u16 + value[Self::ADDR_LEN + 1] as u16 * 256;

        Ok(Self(SocketAddrV6::new(Ipv6Addr::from(addr), port, 0, 0)))
    }
}

impl Wire for Peer {
    fn serialize(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(Self::LEN);
        v.extend_from_slice(&self.0.ip().octets());
        v.push(self.0.port() as u8);
        v.push((self.0.port() >> 8) as u8);
        v
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Peer::try_from(data)?)
    }

    fn len(header: Option<&Header>) -> usize {
        Peer::LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let addr = "[::ffff:255.254.253.252]:7075";
        let peer = Peer::try_from(addr).unwrap();
        let v = peer.serialize();
        let peer2 = Peer::try_from(v.as_slice()).unwrap();
        let addr2 = peer2.socket_addr_v6().to_string();
        assert_eq!(addr, addr2);
    }
}
