use crate::cli::StringOrStdin;
use crate::keys::armor::Armor;
use crate::paths::PathsOpts;
use crate::wallet::{Wallet, WalletId, WalletManager};
use crate::Phrase;
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
                    println!("{}", wallet_id);
                }
                CreateType::Seed(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    manager.add_random_seed(wallet_id.to_owned()).await?;
                    println!("{}", wallet_id);
                }
                CreateType::Private(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    manager.add_random_private(wallet_id.to_owned()).await?;
                    println!("{}", wallet_id);
                }
            },
            Command::Import(o) => match &o.create_type {
                ImportType::Phrase(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    let phrase = Phrase::from_words(
                        o.language.language.to_owned(),
                        o.words.to_owned().resolve()?.as_str(),
                    )?;
                    let wallet = Wallet::Phrase(phrase);
                    manager.add(wallet_id.to_owned(), wallet).await?;
                    println!("{}", wallet_id);
                }
                ImportType::Seed(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    let wallet = Wallet::Seed(o.seed.to_owned().resolve()?);
                    manager.add(wallet_id.to_owned(), wallet).await?;
                    println!("{}", wallet_id);
                }
                ImportType::Private(o) => {
                    let (manager, wallet_id) = WalletOpts::create(&o.opts).await?;
                    let wallet = Wallet::Private(o.private.to_owned().resolve()?);
                    manager.add(wallet_id.to_owned(), wallet).await?;
                    println!("{}", wallet_id);
                }
            },
            Command::Delete(o) => {
                let (manager, wallet_id) = WalletOpts::delete(&o.opts).await?;
                manager.delete(&wallet_id).await?;
                println!("Wallet {:?} was deleted", wallet_id);
            }
            Command::Private(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                println!("{}", wallet.private(o.address)?);
            }
            Command::Public(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                println!("{}", wallet.public(o.address)?);
            }
            Command::Address(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                println!("{}", wallet.address(o.address)?);
            }
            Command::Sign(o) => {
                let wallet = WalletOpts::read(&o.opts).await?;
                let string = o.message.to_owned().resolve()?;
                let message = string.as_bytes();
                let signed = wallet.private(o.address)?.sign(message)?;
                if o.armor {
                    println!("{}", Armor::new(string, wallet.address(o.address)?, signed));
                } else {
                    println!("{}", signed);
                }
            }
        };
        Ok(())
    }

    async fn read(o: &CommonOpts) -> anyhow::Result<Wallet> {
        let manager = WalletManager::new(&o.paths_opts.wallet_path()?);
        let wallet = manager.wallet(&o.wallet_id()?).await?;
        Ok(wallet)
    }

    async fn create(o: &CommonOptsCreate) -> anyhow::Result<(WalletManager, WalletId)> {
        let manager = WalletManager::new(&o.common_opts.paths_opts.wallet_path()?);
        manager.ensure().await?;
        let wallet_id = o.wallet_id()?.to_owned();
        Ok((manager, wallet_id))
    }

    async fn delete(o: &CommonOpts) -> anyhow::Result<(WalletManager, WalletId)> {
        let manager = WalletManager::new(&o.paths_opts.wallet_path()?);
        manager.ensure().await?;
        let wallet_id = o.wallet_id()?;
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

    /// Sign a message using a key in this wallet.
    Sign(SignOpts),

    /// Delete an existing wallet.
    Delete(DeleteOpts),
}

#[derive(Clap)]
struct CommonOpts {
    #[clap(flatten)]
    paths_opts: PathsOpts,

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
    Phrase(ImportPhraseOpts),
    Seed(ImportSeedOpts),
    Private(ImportPrivateOpts),
}

#[derive(Clap)]
struct ImportPhraseOpts {
    words: StringOrStdin<String>,

    #[clap(flatten)]
    pub(crate) language: crate::cli::phrase::LanguageOpt,

    #[clap(flatten)]
    opts: CommonOptsCreate,
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
    address: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}

#[derive(Clap)]
struct PublicOpts {
    #[clap(default_value = "0")]
    address: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}

#[derive(Clap)]
struct AddressOpts {
    #[clap(default_value = "0")]
    address: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}

#[derive(Clap)]
struct DeleteOpts {
    #[clap(flatten)]
    opts: CommonOpts,
}

#[derive(Clap)]
struct SignOpts {
    message: StringOrStdin<String>,

    /// Use the feeless armor format which includes the address, message and signature.
    #[clap(long)]
    armor: bool,

    #[clap(short, long, default_value = "0")]
    address: u32,

    #[clap(flatten)]
    opts: CommonOpts,
}
