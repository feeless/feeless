use crate::cli::pcap::PcapDumpOpts;
use crate::cli::unit::UnitOpts;
use crate::cli::vanity::VanityOpts;
use crate::cli::wallet::WalletOpts;
use crate::debug::parse_pcap_log_file_to_csv;
use crate::node::node_with_autodiscovery;
use address::AddressOpts;
use anyhow::anyhow;
use clap::Clap;
use phrase::PhraseOpts;
use private::PrivateOpts;
use public::PublicOpts;
use seed::SeedOpts;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

mod address;
mod pcap;
mod phrase;
mod private;
mod public;
mod seed;
mod unit;
mod vanity;
mod wallet;

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
    /// Launches a node
    Node(NodeOpts),

    /// Conversion between units, e.g. Rai to Nano
    Unit(UnitOpts),

    /// Manage wallet files.
    Wallet(WalletOpts),

    /// Word mnemonic phrase generation and conversion.
    Phrase(PhraseOpts),

    /// 64 bit seed generation and conversion.
    Seed(SeedOpts),

    /// Private key generation and conversion.
    Private(PrivateOpts),

    /// Public key conversion.
    Public(PublicOpts),

    /// Address conversion.
    Address(AddressOpts),

    /// Find a secret that can generate a custom vanity address.
    Vanity(VanityOpts),

    /// Tool to analyse network capture dumps for Nano packets.
    Pcap(PcapDumpOpts),

    /// Debugging and experimental tools
    Debug(DebugOpts),
}

#[derive(Clap)]
struct NodeOpts {
    /// Comma separated list of IP:PORT pairs. Overrides default initial nodes.
    #[clap(short, long)]
    override_peers: Option<Vec<String>>,
}

#[derive(Clap)]
struct DebugOpts {
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
        Command::Node(o) => node_with_autodiscovery(o.override_peers).await,
        #[cfg(not(feature = "node"))]
        Command::Node(_) => panic!("Compile with the `node` feature to enable this."),

        #[cfg(feature = "pcap")]
        Command::Pcap(o) => o.handle().await,
        #[cfg(not(feature = "pcap"))]
        Command::Pcap(o) => panic!("Compile with the `pcap` feature to enable this."),

        Command::Debug(debug) => match debug.command {
            DebugCommand::PcapLogToCSV(huh) => parse_pcap_log_file_to_csv(&huh.src, &huh.dst),
        },

        Command::Wallet(wallet) => wallet.handle().await,
        Command::Seed(seed) => seed.handle(),
        Command::Private(private) => private.handle(),
        Command::Public(public) => public.handle(),
        Command::Phrase(phrase) => phrase.handle(),
        Command::Address(address) => address.handle(),
        Command::Unit(unit) => unit.handle(),
        Command::Vanity(vanity) => vanity.handle().await,
    }
}

/// The a `T` or the String "-" if reading from stdin.
///
/// Use `resolve()` to turn the enum into `T` by maybe reading from stdin.
#[derive(Copy, Clone)]
enum StringOrStdin<T>
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
