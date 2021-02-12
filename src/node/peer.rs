use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{expect_len, hex_formatter};
use std::net::{Ipv6Addr, SocketAddrV6};
use std::str::FromStr;

pub struct Peer(SocketAddrV6);

impl Peer {
    pub const LEN: usize = 18;
    pub const ADDR_LEN: usize = 16;

    pub fn socket_addr_v6(&self) -> SocketAddrV6 {
        self.0
    }
}

impl FromStr for Peer {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Peer(SocketAddrV6::from_str(s)?))
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
        expect_len(data.len(), Self::len(None)?, "Peer")?;

        let mut addr: [u8; Self::ADDR_LEN] = [0u8; Self::ADDR_LEN];
        addr.copy_from_slice(&data[0..Self::ADDR_LEN]);
        let port: u16 = data[Self::ADDR_LEN] as u16 + data[Self::ADDR_LEN + 1] as u16 * 256;

        Ok(Self(SocketAddrV6::new(Ipv6Addr::from(addr), port, 0, 0)))
    }

    fn len(_header: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Peer::LEN)
    }
}

impl std::fmt::Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Peer({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let addr = "[::ffff:255.254.253.252]:7075";
        let peer = Peer::from_str(addr).unwrap();
        let v = peer.serialize();
        let peer2 = Peer::deserialize(None, v.as_slice()).unwrap();
        let addr2 = peer2.socket_addr_v6().to_string();
        assert_eq!(addr, addr2);
    }
}
