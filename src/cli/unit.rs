use crate::cli::StringOrStdin;
use crate::units::{Mnano, Nano, UnboundedRaw};
use clap::Clap;
use std::str::FromStr;

macro_rules! raw {
    ($dst:expr) => {
        UnboundedRaw::from_str(&$dst.resolve()?)?
    };
}

macro_rules! nano {
    ($dst:expr) => {
        Nano::from_str(&$dst.resolve()?)?
    };
}

macro_rules! mnano {
    ($dst:expr) => {
        Mnano::from_str(&$dst.resolve()?)?
    };
}

#[derive(Clap)]
pub(crate) struct UnitOpts {
    #[clap(subcommand)]
    command: SrcUnit,
}

impl UnitOpts {
    pub fn handle(&self) -> anyhow::Result<()> {
        // This is a bit unwieldy because of the "from to amount" structure.
        // Not sure how to improve.
        let src = match &self.command {
            SrcUnit::Raw(src) => match &src.dst {
                DstCommand::Raw(dst) => raw!(dst).to_unbounded_raw().to_string(),
                DstCommand::Nano(dst) => raw!(dst).to_nano().to_string(),
                DstCommand::Mnano(dst) => raw!(dst).to_mnano().to_string(),
            }
            .to_string(),
            SrcUnit::Nano(src) => match &src.dst {
                DstCommand::Raw(dst) => nano!(dst).to_unbounded_raw().to_string(),
                DstCommand::Nano(dst) => nano!(dst).to_nano().to_string(),
                DstCommand::Mnano(dst) => nano!(dst).to_mnano().to_string(),
            }
            .to_string(),
            SrcUnit::Mnano(src) => match &src.dst {
                DstCommand::Raw(dst) => mnano!(dst).to_unbounded_raw().to_string(),
                DstCommand::Nano(dst) => mnano!(dst).to_nano().to_string(),
                DstCommand::Mnano(dst) => mnano!(dst).to_mnano().to_string(),
            }
            .to_string(),
        };
        println!("{}", src);
        Ok(())
    }
}

#[derive(Clap)]
enum SrcUnit {
    /// From a raw amount.
    Raw(Dst),

    /// From a Nano amount
    Nano(Dst),

    /// From a Cents amount
    Mnano(Dst),
}

#[derive(Clap)]
struct Dst {
    #[clap(subcommand)]
    dst: DstCommand,
}

#[derive(Clap)]
enum DstCommand {
    /// Convert to a raw amount.
    Raw(Opts),

    /// Convert to a nano amount.
    Nano(Opts),

    /// Convert to a Mnano/NANO/Nano amount
    Mnano(Opts),
}

#[derive(Clap)]
struct Opts {
    amount: StringOrStdin<String>,
}

impl Opts {
    fn resolve(&self) -> anyhow::Result<String> {
        self.amount.to_owned().resolve()
    }
}
