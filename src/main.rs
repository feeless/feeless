#![forbid(unsafe_code)]

use clap::Clap;
use feeless::node::node_with_single_peer;
use feeless::pcap;
use feeless::pcap::{PcapDump, Subject};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    Node(NodeOpts),
    Pcap(PcapDumpArgs),
}

#[derive(Clap)]
struct NodeOpts {
    address: String,
}

#[derive(Clap)]
struct PcapDumpArgs {
    path: String,
    subject: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    let result = match opts.command {
        Command::Node(o) => node_with_single_peer(&o.address).await,
        Command::Pcap(o) => {
            let subject = match o.subject {
                Some(ip_addr) => Subject::Specified(Ipv4Addr::from_str(&ip_addr).unwrap()),
                None => Subject::AutoFirstSource,
            };
            let mut p = PcapDump::new(subject);
            p.dump(&o.path)
        }
    };
    if result.is_err() {
        println!("{:#?}", result.unwrap());
    }
}
