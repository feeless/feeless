#![forbid(unsafe_code)]

mod connection;
mod cookie;
mod header;
mod message;
mod state;

use crate::header::{Flags, Header, MessageType, Network};
use crate::state::State;

use connection::Connection;
use std::time::Duration;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let state = State::new();

    let state_clone = state.clone();
    tokio::spawn(async {
        let address = "localhost:7075";
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut peer_handler = Connection::new(state_clone, Network::Live, stream);
        peer_handler.run().await.unwrap();
    });

    let state_clone = state.clone();
    tokio::spawn(async {
        let address = "localhost:7075";
        let stream = TcpStream::connect(&address).await.unwrap();
        // dbg!(stream.peer_addr().unwrap());
        let peer_handler = Connection::new(state_clone, Network::Live, stream);
    });

    println!("Sleeping hax");
    tokio::time::sleep(Duration::from_secs(99999999)).await;
}
