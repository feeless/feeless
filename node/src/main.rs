#![forbid(unsafe_code)]

mod cookie;
mod header;
mod messages;
mod peer;
mod state;
mod wire;

use crate::header::Network;
use crate::state::State;

use peer::Peer;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let state = State::new(Network::Live);

    let state_clone = state.clone();
    let handle = tokio::spawn(async {
        let address = "localhost:7075";
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut peer_handler = Peer::new(state_clone, stream);
        peer_handler.run().await.unwrap();
    });

    println!("Waiting...");
    handle.await.unwrap();
    println!("Quitting...");
}
