use crate::cli::StringOrStdin;
use crate::units::{Cents, MicroNano, Nano, UnboundedRai};
use crate::Rai;
use clap::Clap;
use std::str::FromStr;

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
                DstCommand::Rai(dst) => UnboundedRai::from_str(&dst.resolve()?)?
                    .to_unbounded_rai()
                    .to_string(),
                DstCommand::Nano(dst) => UnboundedRai::from_str(&dst.resolve()?)?
                    .to_nano()
                    .to_string(),
                DstCommand::Cents(dst) => UnboundedRai::from_str(&dst.resolve()?)?
                    .to_cents()
                    .to_string(),
                DstCommand::Micro(dst) => UnboundedRai::from_str(&dst.resolve()?)?
                    .to_micro_nano()
                    .to_string(),
            }
            .to_string(),
            SrcUnit::Nano(src) => match &src.dst {
                DstCommand::Rai(dst) => Nano::from_str(&dst.resolve()?)?
                    .to_unbounded_rai()
                    .to_string(),
                DstCommand::Nano(dst) => Nano::from_str(&dst.resolve()?)?.to_nano().to_string(),
                DstCommand::Cents(dst) => Nano::from_str(&dst.resolve()?)?.to_cents().to_string(),
                DstCommand::Micro(dst) => {
                    Nano::from_str(&dst.resolve()?)?.to_micro_nano().to_string()
                }
            }
            .to_string(),
            SrcUnit::Cents(src) => match &src.dst {
                DstCommand::Rai(dst) => Cents::from_str(&dst.resolve()?)?
                    .to_unbounded_rai()
                    .to_string(),
                DstCommand::Nano(dst) => Cents::from_str(&dst.resolve()?)?.to_nano().to_string(),
                DstCommand::Cents(dst) => Cents::from_str(&dst.resolve()?)?.to_cents().to_string(),
                DstCommand::Micro(dst) => Cents::from_str(&dst.resolve()?)?
                    .to_micro_nano()
                    .to_string(),
            }
            .to_string(),
            SrcUnit::Micro(src) => match &src.dst {
                DstCommand::Rai(dst) => MicroNano::from_str(&dst.resolve()?)?
                    .to_unbounded_rai()
                    .to_string(),
                DstCommand::Nano(dst) => {
                    MicroNano::from_str(&dst.resolve()?)?.to_nano().to_string()
                }
                DstCommand::Cents(dst) => {
                    MicroNano::from_str(&dst.resolve()?)?.to_cents().to_string()
                }
                DstCommand::Micro(dst) => MicroNano::from_str(&dst.resolve()?)?
                    .to_micro_nano()
                    .to_string(),
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
