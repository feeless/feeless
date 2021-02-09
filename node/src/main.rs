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
use clap::Clap;
use tokio::net::TcpStream;

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
        Command::Dump(o) => dump(&o.path).await.unwrap(),
    }
}

async fn node_with_single_peer(address: &str) -> anyhow::Result<()> {
    let state = Box::new(SledState::new(Network::Live));
    let address = address.to_owned();

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

async fn dump(path: &str) -> anyhow::Result<()> {
    dbg!(path);
    Ok(())
}
