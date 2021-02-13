#![forbid(unsafe_code)]

use clap::Clap;
use feeless::node::node_with_single_peer;
use feeless::pcap;

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
    source: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    let result = match opts.command {
        Command::Node(o) => node_with_single_peer(&o.address).await,
        Command::Dump(o) => pcap::pcap_dump(&o.path, &o.source).await,
    };
    if result.is_err() {
        println!("{:#?}", result.unwrap());
    }
}
