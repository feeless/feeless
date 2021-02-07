#![forbid(unsafe_code)]

mod bytes;
mod channel;
mod cookie;
mod header;
mod messages;
mod peer;
mod state;
mod wire;

use crate::header::Network;
use crate::state::SledState;

use channel::Channel;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Box::new(SledState::new(Network::Live));

    let state_clone = state.clone();
    let handle = tokio::spawn(async {
        let address = "localhost:7075";
        let address = "213.136.90.96:7075";
        let stream = TcpStream::connect(&address).await.unwrap();
        let mut channel = Channel::new(state_clone, stream);
        channel.run().await.unwrap();
    });

    println!("Waiting...");
    handle.await.unwrap();
    println!("Quitting...");
}

/*
[::ffff:213.136.90.96]:7075
[::ffff:194.13.81.185]:7075
[::ffff:195.72.210.29]:7075
[::ffff:195.154.160.221]:7075
**/
