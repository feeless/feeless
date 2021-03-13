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

pub use state::{MemoryState, SledDiskState};
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
    let address = address.to_owned();

    // TODO: peering.nano.org

    let _state_clone = Arc::clone(&state);
    info!("Spawning a channel to {}", &address);
    let handle = tokio::spawn(async move {
        let stream = TcpStream::connect(&address).await.unwrap();
        // let mut channel = Channel::new(network, state_clone, stream).await;
        // channel.run().await.unwrap();
        network_channel(network, state, stream)
            .await
            .expect("Error in network_channel");
    });

    handle.await.unwrap();
    info!("Quitting...");
    Ok(())
}
