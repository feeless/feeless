#[cfg(feature = "pcap")]
mod pcap;

mod address;
mod phrase;
mod private;
mod public;
mod seed;
mod unit;
mod vanity;
mod verify;
mod wallet;
mod work;

#[cfg(feature = "rpc_client")]
use crate::rpc::client::RPCClientOpts;

#[cfg(feature = "pcap")]
use crate::cli::pcap::PcapDumpOpts;

#[cfg(feature = "node")]
use crate::node::Node;

use crate::cli::unit::UnitOpts;
use crate::cli::vanity::VanityOpts;
use crate::cli::verify::VerifyOpts;
use crate::cli::wallet::WalletOpts;
use crate::cli::work::WorkOpts;
use address::AddressOpts;
use anyhow::anyhow;
use clap::Clap;
use phrase::PhraseOpts;
use private::PrivateOpts;
use public::PublicOpts;
use seed::SeedOpts;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, io};
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[derive(Clap)]
#[clap(author, about, version)]
struct Opts {
    #[clap(subcommand)]
    command: Command,

    /// Don't use ANSI color codes when logging.
    #[clap(long)]
    no_color: bool,

    /// Maximum level of logging to be displayed: trace, debug, info, warn, error.
    #[clap(short = 'l', long)]
    log_level: Option<Level>,
}

#[derive(Clap)]
enum Command {
    #[cfg(feature = "node")]
    /// Launches a node
    Node(NodeOpts),
    #[cfg(not(feature = "node"))]
    /// Launches a node (DISABLED)
    Node,

    /// Conversion between units, e.g. Raw to Nano
    Unit(UnitOpts),

    /// Manage wallet files.
    Wallet(WalletOpts),

    /// Verify Nano signed messages.
    Verify(VerifyOpts),

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

    /// Generate proof of work.
    Work(WorkOpts),

    /// Find a secret that can generate a custom vanity address.
    Vanity(VanityOpts),

    #[cfg(feature = "rpc_client")]
    /// RPC client that can call a function against a Nano RPC server.
    Call(RPCClientOpts),
    #[cfg(not(feature = "rpc_client"))]
    /// RPC client that can call a function against a Nano RPC server. (DISABLED)
    Call,

    #[cfg(feature = "pcap")]
    /// Tool to analyse network capture dumps for Nano packets.
    Pcap(PcapDumpOpts),
    #[cfg(not(feature = "pcap"))]
    /// Tool to analyse network capture dumps for Nano packets. (DISABLED)
    Pcap,
}

#[derive(Clap)]
struct NodeOpts {
    /// Comma separated list of IP:PORT pairs. Overrides default initial nodes.
    #[clap(short, long)]
    override_peers: Option<Vec<String>>,
}

#[derive(Clap)]
struct PcapLogToCsvArgs {
    src: PathBuf,
    dst: PathBuf,
}

pub async fn run() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let mut filter = EnvFilter::from_default_env();
    if let Some(level) = opts.log_level {
        filter = filter.add_directive(level.into());
    } else if env::var_os("RUST_LOG").is_none() {
        filter = filter.add_directive("feeless=info".parse()?);
    }
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_ansi(!opts.no_color)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not initialize logger");

    match opts.command {
        #[cfg(feature = "node")]
        Command::Node(o) => Node::start(o.override_peers).await,
        #[cfg(not(feature = "node"))]
        Command::Node => panic!("Compile with the `node` feature to enable this."),

        #[cfg(feature = "pcap")]
        Command::Pcap(o) => o.handle().await,
        #[cfg(not(feature = "pcap"))]
        Command::Pcap => panic!("Compile with the `pcap` feature to enable this."),

        #[cfg(feature = "rpc_client")]
        Command::Call(o) => Ok(o.handle().await?),
        #[cfg(not(feature = "rpc_client"))]
        Command::Call => panic!("Compile with the `rpc_client` feature to enable this."),

        Command::Wallet(wallet) => wallet.handle().await,
        Command::Seed(seed) => seed.handle(),
        Command::Private(private) => private.handle(),
        Command::Public(public) => public.handle(),
        Command::Phrase(phrase) => phrase.handle(),
        Command::Address(address) => address.handle(),
        Command::Unit(unit) => unit.handle(),
        Command::Work(work) => work.handle(),
        Command::Vanity(vanity) => vanity.handle().await,
        Command::Verify(verify) => verify.handle(),
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
