use crate::node::channel::Channel;
use crate::node::state::SledState;
use crate::node::wire::header::Network;
use tokio::net::TcpStream;

mod channel;
pub mod messages;
pub mod state;
pub mod wire;

pub async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
    let state = Box::new(SledState::new(Network::Live));
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
