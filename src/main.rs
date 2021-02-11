#![forbid(unsafe_code)]

use clap::Clap;
use feeless::dump;
use feeless::node::node_with_single_peer;




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
