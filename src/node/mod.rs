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
use anyhow::Context;
pub use state::{MemoryState, SledDiskState};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::info;
pub use wire::Wire;

pub async fn node_with_autodiscovery(
    addresses_override: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let network = Network::Live;
    // let state = SledDiskState::new(Network::Live);
    let state = MemoryState::new(network);

    let state = Arc::new(Mutex::new(state));
    let configured_peers = if addresses_override.is_some() {
        parse_socket_list(addresses_override.unwrap())?
    } else {
        // TODO: Make this different depending on network, e.g. `network.peering_host()`
        tokio::net::lookup_host("peering.nano.org:7075")
            .await
            .context("Error while trying to lookup default peers")?
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

fn parse_socket_list(socket_list: Vec<String>) -> Result<Vec<SocketAddr>, anyhow::Error> {
    let mut retval: Vec<SocketAddr> = Vec::new();
    for socket in socket_list {
        let socket = SocketAddr::from_str(socket.as_str())
            .with_context(|| format!("Could not parse correctly {} as SocketAddr", socket))?;
        retval.push(socket)
    }
    Ok(retval)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_socket_list_test() -> Result<(), anyhow::Error> {
        let list = vec!["1.2.3.4:4321".to_string(), "5.4.3.2:9876".to_string()];
        let sockets = parse_socket_list(list)?;
        let socket_under_test = sockets[1];
        assert!(socket_under_test.is_ipv4() && socket_under_test.port() == 9876);
        Ok(())
    }

    #[test]
    fn parse_socket_list_should_fail_when_one_address_is_malformed() {
        let list = vec!["1.2.3.4:7676".to_string(), "5.4.3.2".to_string()];
        assert!(
            parse_socket_list(list).is_err(),
            "should fail because of missing port"
        )
    }
}
