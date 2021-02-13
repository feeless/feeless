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

    /// The IP address of the subject, to show relative information.
    /// This is inferred automatically by the host of the first packet sent out.
    #[clap(short, long)]
    my_addr: Option<String>,

    /// Only show packets related to this IP address.
    #[clap(long)]
    filter_addr: Option<String>,

    #[clap(long)]
    start: Option<usize>,

    #[clap(long)]
    end: Option<usize>,

    /// Show packets over multiple lines.
    #[clap(short, long)]
    expanded: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    let result = match opts.command {
        Command::Node(o) => node_with_single_peer(&o.address).await,
        Command::Pcap(o) => {
            let subject = match o.my_addr {
                Some(ip_addr) => {
                    Subject::Specified(Ipv4Addr::from_str(&ip_addr).expect("a valid ip address"))
                }
                None => Subject::AutoFirstSource,
            };
            let mut p = PcapDump::new(subject);
            p.expanded = o.expanded;
            p.start_at = o.start;
            p.end_at = o.end;
            p.filter_addr = o
                .filter_addr
                .as_ref()
                .map(|i| Ipv4Addr::from_str(i).expect("a valid ip address"));
            p.dump(&o.path)
        }
    };
    if result.is_err() {
        println!("{:#?}", result.unwrap());
    }
}
