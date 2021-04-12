use crate::blocks::{deserialize_to_unsure_link, BlockType, StateBlock};
use crate::blocks::{BlockHash, Link, Subtype};
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::version::Version;
use crate::{Address, Rai, Result, Signature, Work};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct PeersRequest {
    /// Returns a list of peers IPv6:port with its node protocol network version and node ID.
    #[clap(short, long)]
    peer_details: bool,
}

#[async_trait]
impl RPCRequest for &PeersRequest {
    type Response = PeersResponse;

    fn action(&self) -> &str {
        "peers"
    }

    async fn call(&self, client: &RPCClient) -> Result<PeersResponse> {
        client.rpc(self).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeersResponse {
    /// The type in peers depends on the value set in [PeersRequest::peer_details].
    peers: Peers,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Peers {
    Simple(Vec<SocketAddr>),
    Details(HashMap<SocketAddr, Peer>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Peer {
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    protocol_version: Version,

    node_id: String, // TODO: NodeId type. It might be used in the node handshake!

    #[serde(rename = "type")]
    net_type: NetType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NetType {
    Tcp,
    Udp,
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV6};
    use std::str::FromStr;

    use super::*;

    #[test]
    fn deserialize_simple() {
        let s = r#"{
            "peers": [
                "[::ffff:172.17.0.1]:32841"
            ]
        }"#;
        let o: PeersResponse = serde_json::from_str(s).unwrap();
        if let Peers::Simple(peers) = o.peers {
            let socket_addr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from_str("::ffff:172.17.0.1").unwrap()),
                32841,
            );
            assert_eq!(peers[0], socket_addr);
        } else {
            assert!(false, "Did not parse a simple list");
        };
    }

    #[test]
    fn deserialize_detail() {
        let s = r#"{
            "peers": {
                "[::ffff:172.17.0.1]:32841": {
                    "protocol_version": "18",
                    "node_id": "node_1y7j5rdqhg99uyab1145gu3yur1ax35a3b6qr417yt8cd6n86uiw3d4whty3",
                    "type": "tcp"
                }
            }
        }"#;
        let o: PeersResponse = serde_json::from_str(s).unwrap();
        if let Peers::Details(peers) = o.peers {
            assert_eq!(peers.len(), 1);
            let socket_addr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from_str("::ffff:172.17.0.1").unwrap()),
                32841,
            );
            let peer = peers.get(&socket_addr).unwrap();
            assert_eq!(
                peer,
                &Peer {
                    protocol_version: Version::V18,
                    node_id: "node_1y7j5rdqhg99uyab1145gu3yur1ax35a3b6qr417yt8cd6n86uiw3d4whty3"
                        .to_string(),
                    net_type: NetType::Tcp
                }
            );
        } else {
            assert!(false, "Did not parse detailed peers");
        };
    }
}
