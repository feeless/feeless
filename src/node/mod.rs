use tokio::net::TcpStream;

use network::Network;

use crate::node::channel::Channel;
use state::SledDiskState;

mod channel;
mod controller;
pub mod cookie;
pub mod header;
pub mod messages;
pub mod network;
pub mod peer;
pub mod state;
pub mod wire;

pub async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
    let state = Box::new(SledDiskState::new(Network::Live));
    let address = address.to_owned();

    // TODO: peering.nano.org

    let state_clone = state.clone();
    let handle = tokio::spawn(async move {
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut channel = Channel::new(state_clone, stream);
        channel.run().await.unwrap();
    });

    println!("Waiting...");
    handle.await.unwrap();
    println!("Quitting...");
    Ok(())
}
