use crate::cli::StringOrStdin;
use crate::wallet::{WalletId, WalletManager};
use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
pub struct WalletOpts {
    #[clap(subcommand)]
    command: Command,
}

impl WalletOpts {
    pub async fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::New(c) => match &c.create_type {
                CreateType::Seed(o) => {
                    let manager = WalletManager::new(&o.opts.file);
                    manager.ensure().await?;
                    let wallet_id = o.opts.wallet_id()?.to_owned();
                    manager.add_random_seed(wallet_id.to_owned()).await?;
                    println!("{:?}", wallet_id);
                }
                CreateType::Private(_) => {}
            },
            Command::Import(_) => {}
            Command::Private(_) => {}
            Command::Public(_) => {}
            Command::Address(_) => {}
        };
        Ok(())
    }
}

#[derive(Clap)]
enum Command {
    /// Create a new wallet. If the wallet file doesn't exist, it will be created.
    New(NewOpts),

    /// Import an existing wallet.
    Import(ImportOpts),

    /// Output the private key of a wallet.
    Private(PrivateOpts),

    /// Output the public address of a wallet.
    Public(PublicOpts),

    /// Output the address of a wallet.
    Address(AddressOpts),
}

#[derive(Clap)]
struct CommonOptsCreate {
    /// Path to the wallet file.
    #[clap(short, long, env = "FEELESS_WALLET_FILE")]
    file: PathBuf,

    /// Wallet ID.
    #[clap(short, long, env = "FEELESS_WALLET_ID")]
    id: Option<WalletId>,

    #[clap(short, long)]
    default: bool,
}

impl CommonOptsCreate {
    fn wallet_id(&self) -> anyhow::Result<WalletId> {
        if self.default {
            return Ok(WalletId::zero());
        }

        if let Some(wallet_id) = &self.id {
            Ok(wallet_id.to_owned())
        } else {
            Ok(WalletId::random())
        }
    }
}

#[derive(Clap)]
struct CreateOpts {
    #[clap(flatten)]
    opts: CommonOptsCreate,
}

#[derive(Clap)]
struct NewOpts {
    #[clap(subcommand)]
    create_type: CreateType,
}

#[derive(Clap)]
enum CreateType {
    Seed(CreateSeedOpts),
    Private(CreatePrivateOpts),
}

#[derive(Clap)]
struct CreateSeedOpts {
    #[clap(flatten)]
    opts: CommonOptsCreate,
}

#[derive(Clap)]
struct CreatePrivateOpts {
    #[clap(flatten)]
    opts: CommonOptsCreate,
}

#[derive(Clap)]
struct ImportOpts {
    #[clap(subcommand)]
    create_type: ImportType,
}

#[derive(Clap)]
enum ImportType {
    Seed(ImportSeedOpts),
    Private(ImportPrivateOpts),
}

#[derive(Clap)]
struct ImportSeedOpts {
    seed: StringOrStdin<crate::Seed>,

    #[clap(flatten)]
    opts: CommonOptsCreate,
}

#[derive(Clap)]
struct ImportPrivateOpts {
    private: StringOrStdin<crate::Private>,

    #[clap(flatten)]
    opts: CommonOptsCreate,
}

#[derive(Clap)]
struct PrivateOpts {}

#[derive(Clap)]
struct PublicOpts {}

#[derive(Clap)]
struct AddressOpts {}
