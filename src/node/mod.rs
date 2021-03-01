use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::info;

use state::SledDiskState;

use crate::network::Network;
use crate::node::channel::network_channel;
use crate::node::state::{DynState, MemoryState};

mod channel;
pub mod controller;
pub mod cookie;
pub mod header;
pub mod messages;
pub mod peer;
pub mod state;
pub mod timestamp;
pub mod wire;

pub async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
    let network = Network::Live;
    // let state = SledDiskState::new(Network::Live);
    let state = MemoryState::new(network);

    let state = Arc::new(Mutex::new(state));
    let address = address.to_owned();

    // TODO: peering.nano.org

    let state_clone = Arc::clone(&state);
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
