#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Clap;
use tokio::net::TcpStream;

use channel::Channel;
use wire::header::Network;

use crate::state::SledState;

mod bytes;
mod channel;
mod dump;
mod messages;
mod state;
mod wire;

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    Node(NodeOpts),
    Dump(DumpArgs),
}

#[derive(Clap)]
struct NodeOpts {
    address: String,
}

#[derive(Clap)]
struct DumpArgs {
    path: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    match opts.command {
        Command::Node(o) => node_with_single_peer(&o.address).await.unwrap(),
        Command::Dump(o) => dump::dump(&o.path).await.unwrap(),
    }
}

async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
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
