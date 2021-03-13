#![forbid(unsafe_code)]

#[cfg(feature = "node")]
use feeless::node::node_with_single_peer;

#[cfg(feature = "pcap")]
use feeless::pcap::{PcapDump, Subject};

use crate::DebugCommand::PcapLogToCSV;
use ansi_term::Color;
use anyhow::Context;
use clap::Clap;
use feeless::cli;
use feeless::cli::convert::ConvertFrom;
use feeless::debug::parse_pcap_log_file_to_csv;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[derive(Clap)]
#[clap(author, about, version)]
struct Opts {
    #[clap(subcommand)]
    command: Command,

    /// Don't use ANSI colour codes when logging.
    #[clap(long)]
    no_color: bool,
}

#[derive(Clap)]
enum Command {
    Node(NodeOpts),

    Convert(ConvertFrom),

    /// Word mnemonic phrase generation and conversion.
    Phrase(cli::Phrase),

    Seed(cli::Seed),

    Private(cli::Private),

    Public(cli::Public),

    Address(cli::Address),

    Pcap(PcapDumpArgs),

    /// Debugging and experimental tools
    Debug(Debug),
}

#[derive(Clap)]
struct NodeOpts {
    address: String,
}

/// Read a pcapng file containing Nano packets, and print some information about each payload.
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

    /// Starting packet.
    #[clap(long)]
    start: Option<usize>,

    /// Last packet to process.
    #[clap(long)]
    end: Option<usize>,
}

#[derive(Clap)]
struct Debug {
    #[clap(subcommand)]
    command: DebugCommand,
}

#[derive(Clap)]
enum DebugCommand {
    PcapLogToCSV(PcapLogToCsvArgs),
}

#[derive(Clap)]
struct PcapLogToCsvArgs {
    src: PathBuf,
    dst: PathBuf,
}

#[tokio::main]
async fn main() {
    let result = option(Opts::parse()).await;
    if let Err(err) = result {
        error!("Exiting because of an error: {:?}", err);
        std::process::exit(1);
    }
}

async fn option(opts: Opts) -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_ansi(!opts.no_color)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not initialize logger");

    match opts.command {
        #[cfg(feature = "node")]
        Command::Node(o) => node_with_single_peer(&o.address).await,
        #[cfg(not(feature = "node"))]
        Command::Node(_) => panic!("Compile with the `node` feature to enable this."),

        #[cfg(feature = "pcap")]
        Command::Pcap(o) => {
            let subject = match o.my_addr {
                Some(ip_addr) => {
                    Subject::Specified(Ipv4Addr::from_str(&ip_addr).context("Invalid IP address")?)
                }
                None => Subject::AutoFirstSource,
            };
            let mut p = PcapDump::new(subject);
            p.start_at = o.start;
            p.end_at = o.end;
            p.filter_addr = o
                .filter_addr
                .as_ref()
                .map(|i| Ipv4Addr::from_str(i).context("Invalid IP address"))
                .transpose()?;
            p.dump(&o.path).await
        }
        #[cfg(not(feature = "pcap"))]
        Command::Pcap(o) => panic!("Compile with the `pcap` feature to enable this."),

        Command::Debug(debug) => match debug.command {
            DebugCommand::PcapLogToCSV(huh) => parse_pcap_log_file_to_csv(&huh.src, &huh.dst),
        },

        Command::Seed(seed) => seed.handle(),
        Command::Private(private) => private.handle(),
        Command::Public(public) => public.handle(),
        Command::Phrase(phrase) => phrase.handle(),
        Command::Convert(from) => from.command.handle(),
        Command::Address(address) => address.handle(),
    }
}
