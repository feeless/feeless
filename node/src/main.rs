#![forbid(unsafe_code)]

mod channel;
mod cookie;
mod header;
mod messages;
mod peer;
mod state;
mod wire;

use crate::header::Network;
use crate::state::State;

use channel::Channel;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = State::new(Network::Live);

    let state_clone = state.clone();
    let handle = tokio::spawn(async {
        let address = "localhost:7075";
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut channel = Channel::new(state_clone, stream);
        channel.run().await.unwrap();
    });

    println!("Waiting...");
    handle.await.unwrap();
    println!("Quitting...");
}
