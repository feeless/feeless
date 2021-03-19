use crate::cli::StringOrStdin;
use crate::wallet::{Wallet, WalletId, WalletManager};
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
                CreateType::Phrase(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    manager
                        .add_random_phrase(
                            wallet_id.to_owned(),
                            o.phrase_opts.words.0.to_owned(),
                            o.phrase_opts.language.language.to_owned(),
                        )
                        .await?;
                    println!("{:?}", wallet_id);
                }
                CreateType::Seed(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    manager.add_random_seed(wallet_id.to_owned()).await?;
                    println!("{:?}", wallet_id);
                }
                CreateType::Private(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    manager.add_random_private(wallet_id.to_owned()).await?;
                    println!("{:?}", wallet_id);
                }
            },
            Command::Import(o) => match &o.create_type {
                ImportType::Seed(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    let wallet = Wallet::Seed(o.seed.to_owned().resolve()?);
                    manager.add(wallet_id.to_owned(), wallet).await?;
                    println!("{:?}", wallet_id);
                }
                ImportType::Private(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    let wallet = Wallet::Private(o.private.to_owned().resolve()?);
                    manager.add(wallet_id.to_owned(), wallet).await?;
                    println!("{:?}", wallet_id);
                }
            },
            Command::Private(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                println!("{}", wallet.private(o.index)?);
            }
            Command::Public(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                println!("{}", wallet.public(o.index)?);
            }
            Command::Address(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                println!("{}", wallet.address(o.index)?);
            }
        };
        Ok(())
    }

    async fn read(o: &CommonOpts) -> anyhow::Result<Wallet> {
        let manager = WalletManager::new(&o.file);
        let wallet = manager.wallet(&o.wallet_id()?).await?;
        Ok(wallet)
    }

    async fn create(o: &CommonOptsCreate) -> anyhow::Result<(WalletManager, WalletId)> {
        let manager = WalletManager::new(&o.common_opts.file);
        manager.ensure().await?;
        let wallet_id = o.wallet_id()?.to_owned();
        Ok((manager, wallet_id))
    }
}

#[derive(Clap)]
enum Command {
    /// Create a new wallet. If the wallet file doesn't exist, it will be created.
    New(NewOpts),

    /// Import an existing wallet. If the wallet file doesn't exist, it will be created.
    Import(ImportOpts),

    /// Output the private key of a wallet.
    Private(PrivateOpts),

    /// Output the public address of a wallet.
    Public(PublicOpts),

    /// Output the address of a wallet.
    Address(AddressOpts),
}

#[derive(Clap)]
struct CommonOpts {
    /// Path to the wallet file.
    #[clap(short, long, env = "FEELESS_WALLET_FILE")]
    file: PathBuf,

    /// Wallet ID.
    #[clap(short, long, env = "FEELESS_WALLET_ID")]
    id: Option<WalletId>,
}

impl CommonOpts {
    fn wallet_id(&self) -> anyhow::Result<WalletId> {
        if let Some(wallet_id) = &self.id {
            Ok(wallet_id.to_owned())
        } else {
            Ok(WalletId::zero())
        }
    }
}

#[derive(Clap)]
struct CommonOptsCreate {
    #[clap(flatten)]
    common_opts: CommonOpts,

    #[clap(short, long)]
    default: bool,
}

impl CommonOptsCreate {
    fn wallet_id(&self) -> anyhow::Result<WalletId> {
        if self.default {
            return Ok(WalletId::zero());
        }

        if let Some(wallet_id) = &self.common_opts.id {
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
    Phrase(CreatePhraseOpts),
    Seed(CreateSeedOpts),
    Private(CreatePrivateOpts),
}

#[derive(Clap)]
struct CreatePhraseOpts {
    #[clap(flatten)]
    pub phrase_opts: super::phrase::New,

    #[clap(flatten)]
    pub opts: CommonOptsCreate,
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
struct PrivateOpts {
    #[clap(default_value = "0")]
    index: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}

#[derive(Clap)]
struct PublicOpts {
    #[clap(default_value = "0")]
    index: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}

#[derive(Clap)]
struct AddressOpts {
    #[clap(default_value = "0")]
    index: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}
