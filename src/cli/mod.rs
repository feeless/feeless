use crate::debug::parse_pcap_log_file_to_csv;
use crate::node::node_with_single_peer;
use crate::pcap;
use address::Address;
use anyhow::{anyhow, Context};
use clap::Clap;
use convert::ConvertFrom;
use phrase::Phrase;
use private::Private;
use public::Public;
use seed::Seed;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::io;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;

mod address;
mod convert;
mod phrase;
mod private;
mod public;
mod seed;

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
    Phrase(Phrase),

    /// 64 bit seed generation and conversion.
    Seed(Seed),

    /// Private key generation and conversion.
    Private(Private),

    /// Public key conversion.
    Public(Public),

    /// Address conversion.
    Address(Address),

    /// Tool to analyse network capture dumps for Nano packets.
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

pub async fn run() -> anyhow::Result<()> {
    let opts = Opts::parse();

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
                Some(ip_addr) => pcap::Subject::Specified(
                    Ipv4Addr::from_str(&ip_addr).context("Invalid IP address")?,
                ),
                None => pcap::Subject::AutoFirstSource,
            };
            let mut p = pcap::PcapDump::new(subject);
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

/// The a `T` or the String "-" if reading from stdin.
///
/// Use `resolve()` to turn the enum into `T` by maybe reading from stdin.
#[derive(Copy, Clone)]
pub enum StringOrStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    String(T),
    Stdin,
}

impl<T> StringOrStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    /// Resolve `T` by reading from stdin if necessary.
    pub fn resolve(self) -> anyhow::Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        match self {
            StringOrStdin::String(t) => Ok(t),
            StringOrStdin::Stdin => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                Ok(T::from_str(buffer.trim())
                    .map_err(|e| anyhow!("Conversion from string failed: {:?}", e))?)
            }
        }
    }
}

impl<T> FromStr for StringOrStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type Err = anyhow::Error;

    // This wasn't done in one step because I think clap calls from_str twice, and the second time
    // around stdin is empty.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_ref() {
            "-" => Ok(StringOrStdin::Stdin),
            x => match T::from_str(x) {
                Ok(x) => Ok(StringOrStdin::String(x)),
                Err(e) => Err(anyhow!("Could not parse string: {:?}", e)),
            },
        }
    }
}
