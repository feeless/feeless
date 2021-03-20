use crate::cli::StringOrStdin;
use crate::units::{Cents, MicroNano, Nano, UnboundedRai};
use clap::Clap;
use std::str::FromStr;

macro_rules! rai {
    ($dst:expr) => {
        UnboundedRai::from_str(&$dst.resolve()?)?
    };
}

macro_rules! nano {
    ($dst:expr) => {
        Nano::from_str(&$dst.resolve()?)?
    };
}

macro_rules! cents {
    ($dst:expr) => {
        Cents::from_str(&$dst.resolve()?)?
    };
}

macro_rules! micro {
    ($dst:expr) => {
        MicroNano::from_str(&$dst.resolve()?)?
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
            SrcUnit::Rai(src) => match &src.dst {
                DstCommand::Rai(dst) => rai!(dst).to_unbounded_rai().to_string(),
                DstCommand::Nano(dst) => rai!(dst).to_nano().to_string(),
                DstCommand::Cents(dst) => rai!(dst).to_cents().to_string(),
                DstCommand::Micro(dst) => rai!(dst).to_micro_nano().to_string(),
            }
            .to_string(),
            SrcUnit::Nano(src) => match &src.dst {
                DstCommand::Rai(dst) => nano!(dst).to_unbounded_rai().to_string(),
                DstCommand::Nano(dst) => nano!(dst).to_nano().to_string(),
                DstCommand::Cents(dst) => nano!(dst).to_cents().to_string(),
                DstCommand::Micro(dst) => nano!(dst).to_micro_nano().to_string(),
            }
            .to_string(),
            SrcUnit::Cents(src) => match &src.dst {
                DstCommand::Rai(dst) => cents!(dst).to_unbounded_rai().to_string(),
                DstCommand::Nano(dst) => cents!(dst).to_nano().to_string(),
                DstCommand::Cents(dst) => cents!(dst).to_cents().to_string(),
                DstCommand::Micro(dst) => cents!(dst).to_micro_nano().to_string(),
            }
            .to_string(),
            SrcUnit::Micro(src) => match &src.dst {
                DstCommand::Rai(dst) => micro!(dst).to_unbounded_rai().to_string(),
                DstCommand::Nano(dst) => micro!(dst).to_nano().to_string(),
                DstCommand::Cents(dst) => micro!(dst).to_cents().to_string(),
                DstCommand::Micro(dst) => micro!(dst).to_micro_nano().to_string(),
            },
        };
        println!("{}", src);
        Ok(())
    }
}

#[derive(Clap)]
enum SrcUnit {
    /// From a rai amount.
    Rai(Dst),

    /// From a Nano amount
    Nano(Dst),

    /// From a Cents amount
    Cents(Dst),

    /// From a MicroNano amount
    Micro(Dst),
}

#[derive(Clap)]
struct Dst {
    #[clap(subcommand)]
    dst: DstCommand,
}

#[derive(Clap)]
enum DstCommand {
    /// Convert to a rai amount.
    Rai(Opts),

    /// Convert to a Nano amount.
    Nano(Opts),

    /// Convert to a Cent amount
    Cents(Opts),

    /// Convert to a MicroNano amount
    Micro(Opts),
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
