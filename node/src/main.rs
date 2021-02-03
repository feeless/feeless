#![forbid(unsafe_code)]

mod connection;
mod cookie;
mod header;
mod message;
mod state;
mod wire;

use crate::header::{Flags, Header, MessageType, Network};
use crate::state::State;

use connection::Connection;
use std::time::Duration;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let state = State::new(Network::Live);

    let state_clone = state.clone();
    let handle = tokio::spawn(async {
        let address = "localhost:7075";
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut peer_handler = Connection::new(state_clone, stream);
        peer_handler.run().await.unwrap();
    });

    println!("Waiting...");
    handle.await;
    println!("Quitting...");
}
