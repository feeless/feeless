mod channel;
mod controller;
mod cookie;
mod header;
mod messages;
mod peer;
mod state;
mod timestamp;
mod wire;

use crate::network::Network;
use channel::network_channel;
pub use controller::{Controller, Packet};
pub use header::Header;

use crate::node::state::State;
pub use state::{MemoryState, SledDiskState};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::info;
pub use wire::Wire;

pub async fn node_with_autodiscovery(addresses_override: Option<String>) -> anyhow::Result<()> {
    let network = Network::Live;
    // let state = SledDiskState::new(Network::Live);
    let state = MemoryState::new(network);

    let state = Arc::new(Mutex::new(state));
    let configured_peers = if addresses_override.is_some() {
        parse_socket_list(addresses_override.unwrap())
    } else {
        tokio::net::lookup_host("peering.nano.org:7075")
            .await
            .unwrap()
            .collect::<Vec<SocketAddr>>()
    };
    state.lock().await.add_peers(configured_peers).await?;

    let mut handles = vec![];
    let initial_peers = state.lock().await.peers().await?;
    for socket_addr in initial_peers {
        info!("Spawning a channel to {}", socket_addr);
        let state = state.clone();
        let handle = tokio::spawn(async move {
            let stream = TcpStream::connect(socket_addr).await.unwrap();
            network_channel(network, state, stream)
                .await
                .expect("Error in network_channel")
        });
        handles.push(handle)
    }

    for handle in handles {
        handle.await?
    }
    info!("Quitting...");
    Ok(())
}

fn parse_socket_list(socket_list: String) -> Vec<SocketAddr> {
    socket_list
        .split(',')
        .map(|s| SocketAddr::from_str(s).unwrap())
        .collect::<Vec<SocketAddr>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_socket_list_test() {
        let list = "1.2.3.4:4321,5.4.3.2:9876";
        let sockets = parse_socket_list(list.to_string());
        let socket_under_test = sockets[1];
        assert!(socket_under_test.is_ipv4() && socket_under_test.port() == 9876)
    }
}
