use crate::phrase::{Language, MnemonicType};
use crate::{Address, Private, Public, Seed};
use anyhow::anyhow;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

/// A reference to a wallet file. **Warning**: Wallet files are not locked (yet).
pub struct WalletManager {
    path: PathBuf,
}

impl WalletManager {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    /// This should be called to create the file if it doesn't exists.
    pub async fn ensure(&self) -> anyhow::Result<()> {
        if self.path.exists() {
            return Ok(());
        }

        let store = WalletStorage::new();
        let file = File::create(&self.path)?;
        serde_json::to_writer_pretty(file, &store)?;

        Ok(())
    }

    /// An internal method for loading the wallet storage.
    ///
    /// TODO: There should be a file lock around this.
    async fn load_unlocked(&self) -> anyhow::Result<(File, WalletStorage)> {
        let file = File::open(&self.path)?;
        let store: WalletStorage = serde_json::from_reader(&file)?;
        Ok((file, store))
    }

    /// An internal method for save the wallet storage.
    ///
    /// TODO: There should be a file lock around this.
    async fn save_unlocked(&self, file: File, store: WalletStorage) -> anyhow::Result<()> {
        Ok(serde_json::to_writer_pretty(file, &store)?)
    }

    pub async fn wallet(&self, reference: &WalletReference) -> anyhow::Result<Wallet> {
        // TODO: File lock
        let (file, store) = self.load_unlocked().await?;
        Ok(store
            .wallets
            .get(&reference)
            .ok_or_else(|| anyhow!("Wallet reference not found: {:?}", &reference))?
            .to_owned())
    }

    pub async fn add_random_phrase(
        &self,
        mnemonic_type: MnemonicType,
        lang: Language,
    ) -> anyhow::Result<Wallet> {
        todo!()
    }

    pub async fn add_random_seed(&self, reference: WalletReference) -> anyhow::Result<Wallet> {
        let wallet = Wallet::new(reference, WalletSecret::Seed(Seed::random()));
        self.add(wallet.clone()).await?;
        Ok(wallet)
    }

    pub async fn add_random_private(&self, reference: WalletReference) -> anyhow::Result<Wallet> {
        let wallet = Wallet::new(reference, WalletSecret::Private(Private::random()));
        self.add(wallet.clone()).await?;
        Ok(wallet)
    }

    /// Add a new wallet to the store.
    ///
    /// If the wallet reference already exists, there will be an error.
    pub async fn add(&self, wallet: Wallet) -> anyhow::Result<()> {
        // TODO: File lock

        let (file, mut storage) = self.load_unlocked().await?;

        if storage.wallets.contains_key(&wallet.reference) {
            return Err(anyhow!(
                "Wallet reference already exists: {:?}",
                &wallet.reference
            ));
        }

        storage.wallets.insert(wallet.reference.clone(), wallet);
        serde_json::to_writer_pretty(file, &storage)?;
        Ok(())
    }
}

/// An individual wallet that can store a different type of seed, etc.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Wallet {
    reference: WalletReference,
    secret: WalletSecret,
}

impl Wallet {
    pub fn new(reference: WalletReference, secret: WalletSecret) -> Wallet {
        Self { reference, secret }
    }

    /// Derive or return a private key for this wallet.
    pub fn private(&self, index: u32) -> anyhow::Result<Private> {
        match &self.secret {
            WalletSecret::Seed(seed) => Ok(seed.derive(index)),
            WalletSecret::Private(private) => {
                if index != 0 {
                    return Err(anyhow!(
                        "There is only one private key in this wallet. Only use index 0."
                    ));
                }
                Ok(private.to_owned())
            }
        }
    }

    /// Derive a public key for this wallet.
    pub fn public(&self, index: u32) -> anyhow::Result<Public> {
        self.private(index)?.to_public()
    }

    /// Derive an address for this wallet.
    pub fn address(&self, index: u32) -> anyhow::Result<Address> {
        Ok(self.public(index)?.to_address())
    }
}

/// The secret of an individual wallet.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WalletSecret {
    /// A wallet that derives keys from a phrase.
    /// TODO: Change `Phrase` so that it can be Serialized
    // Phrase(Phrase),

    /// A wallet that derives from a seed.
    Seed(Seed),

    /// A wallet with a single private key.
    Private(Private),
}

impl WalletSecret {}

/// Storage for all wallets.
#[derive(Serialize, Deserialize)]
pub struct WalletStorage {
    wallets: HashMap<WalletReference, Wallet>,
}

impl WalletStorage {
    pub fn new() -> Self {
        Self {
            wallets: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Debug)]
pub enum WalletReference {
    Default,
    Id(WalletId),
}

/// A unique identifier for a wallet. This can be generated randomly and given to the user for
/// future reference, or given by the user.
#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Debug, Clone)]
pub struct WalletId([u8; WalletId::LEN]);

impl WalletId {
    pub(crate) const LEN: usize = 32;

    fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn random() -> Self {
        let mut id = Self::zero();
        rand::thread_rng().fill_bytes(&mut id.0);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phrase::{Language, MnemonicType};
    use std::str::FromStr;

    #[tokio::test]
    async fn sanity_check() {
        let manager = WalletManager::new("test.wallet");
        manager.ensure().await.unwrap();
        let w1 = manager
            .add_random_seed(WalletReference::Default)
            .await
            .unwrap();

        let w2 = manager.wallet(&WalletReference::Default).await.unwrap();
        assert_eq!(w1.address(0).unwrap(), w2.address(0).unwrap())
    }

    #[tokio::test]
    async fn import_seed() {
        let manager = WalletManager::new("import_seed.wallet");
        manager.ensure().await.unwrap();
        let seed =
            Seed::from_str("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        let secret = WalletSecret::Seed(seed);
        let reference = WalletReference::Id(WalletId::zero());
        manager.add(Wallet::new(reference, secret)).await.unwrap();
    }

    // Just mulling over the API and CLI interaction...
    // feeless wallet create --file gak.wallet
    // FEELESS_WALLET=gak.wallet feeless wallet create
    // Should warn user if wallet exists:
    // * Warning: File already exists with (3) wallets.
    // let manager = WalletManager::new("test.wallet");
    // manager.create().await?; // Fail if already exists
    // manager.ensure().await?; // Will create if doesn't exist

    /*
    fn ensure() {
        let _lock = self.lock().await?;
        if !self.exists() {
            self.save_unlocked().await?;
        }
    }
    */

    // feeless wallet new phrase --file gak.wallet --language en --words 24
    // stdout is the wallet id:
    // A1B2C3....
    //
    // feeless wallet new phrase --default --file etc
    // --default can be used as the default wallet, so that you dont need to track wallet_ids
    // or WALLET_ID=default
    // let wallet = manager.new_phrase(Language::English, 24).await.unwrap();

    // feeless wallet import phrase "banana cat" --default
    // feeless wallet import phrase "banana cat" --id A1B2
    // feeless wallet import phrase "banana cat" # none specified generates a new wallet id
    // feeless wallet import private "a1b2c3"
    // feeless wallet import seed -
    /*
    fn new_phrase(&self, lang, words) {
        let _lock = manager.lock();
        let phrase = Phrase::new(lang, words);
        let wallet_id = self.generate_id();
        let store = self.load_unlocked();
        // TODO: Make sure the wallet id doesnt exist yet
        let s = serde_json::to_string(phrase);
        store.insert(wallet_id, s);
        self.save_unlocked(store);
    }
     */

    // let wallet = manager.new_seed().await.unwrap();
    // let wallet = manager.from_seed(Seed::from_str("A1B2C3").unwrap()).await.unwrap();
    // let wallet = manager.new_private().await.unwrap();
    // let wallet = manager.from_private().await.unwrap();

    // feeless wallet address 42 --id A1B2C3 --path ...
    // nano_1abc
    // feeless wallet address 42
    // Sorry no "default" wallet has been set up. Please use --default when creating a wallet.

    // FEELESS_WALLET_PATH=gak.wallet
    // FEELESS_WALLET_ID=a1b2c3
    // feeless wallet address 42
    // feeless wallet address # 0 is the default
    // feeless wallet address 32

    // Get a wallet
    // let wallet = manager.wallet(wallet_id);
    // wallet is read only and not locked? no need to write to it once we have the seed.

    // let private = wallet.private(0).unwrap();
    // let private = wallet.public(0).unwrap();
    // let address = wallet.address(0).unwrap();
    // let signature = wallet.address(0).sign("hello").unwrap();

    // let wallet_id = wallet_file.new_seed().await.unwrap();
    // wallet.address(wallet_id)
}
