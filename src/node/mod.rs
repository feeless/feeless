use crate::node::channel::Channel;
use crate::node::state::{DynState, MemoryState};
use network::Network;
use state::SledDiskState;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::info;

mod channel;
pub mod controller;
pub mod cookie;
pub mod header;
pub mod messages;
pub mod network;
pub mod peer;
pub mod state;
pub mod timestamp;
pub mod wire;

pub async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
    // let state = SledDiskState::new(Network::Live);
    let state = MemoryState::new(Network::Live);

    let state = Arc::new(Mutex::new(state));
    let address = address.to_owned();

    // TODO: peering.nano.org

    let state_clone = Arc::clone(&state);
    info!("Spawning a channel to {}", &address);
    let handle = tokio::spawn(async move {
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut channel = Channel::new(state_clone, stream).await;
        channel.run().await.unwrap();
    });

    handle.await.unwrap();
    info!("Quitting...");
    Ok(())
}
