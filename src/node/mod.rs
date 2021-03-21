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

pub async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
    let network = Network::Live;
    // let state = SledDiskState::new(Network::Live);
    let state = MemoryState::new(network);

    let state = Arc::new(Mutex::new(state));
    let address = SocketAddr::from_str(address).unwrap();
    state.lock().await.add_peer(address).await?;

    let preconfigured_peers = tokio::net::lookup_host("peering.nano.org:7075")
        .await
        .unwrap();
    for socket_addr in preconfigured_peers {
        let mut state = state.lock().await;
        state.add_peer(socket_addr).await?;
    }

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
