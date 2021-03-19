//! File storage for seeds and private keys.
//!
//! # Manager
//! A [WalletManager] is provided to store multiple [Wallet]s of different types. The supported
//! wallets are [Wallet::Seed], [Wallet::Private], and (TODO) [Wallet::Phrase].
//!
//! ## Example usage
//! ```
//! use feeless::wallet::WalletManager;
//!
//! async fn main() -> anyhow::Result<()> {
//! let manager = WalletManager::new("my.wallet").await?;
//! Ok(())
//! }
//! ```

use crate::phrase::{Language, MnemonicType};
use crate::{to_hex, Address, Private, Public, Seed};
use anyhow::anyhow;
use rand::RngCore;
use serde::de::{Error};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};

use std::path::PathBuf;

use tokio::fs::{File};

/// Manages multiple [Wallet]s of different types of [Wallet]s. **Warning**: Wallet files are not
/// locked (yet).
///
/// There is a concept of a "default" wallet which is a [WalletId] of zeros. This wallet is a
/// wallet that just needs to be used by a user without having to track a random [WalletId].
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
        let file = File::create(&self.path).await?;
        serde_json::to_writer_pretty(file.into_std().await, &store)?;

        Ok(())
    }

    /// An internal method for loading the wallet storage.
    ///
    /// TODO: There should be a file lock around this.
    async fn load_unlocked(&self) -> anyhow::Result<WalletStorage> {
        let file = File::open(&self.path).await?;
        let store: WalletStorage = serde_json::from_reader(&file.into_std().await)?;
        Ok(store)
    }

    /// An internal method for save the wallet storage.
    ///
    /// TODO: There should be a file lock around this.
    async fn save_unlocked(&self, file: File, store: WalletStorage) -> anyhow::Result<()> {
        Ok(serde_json::to_writer_pretty(file.into_std().await, &store)?)
    }

    pub async fn wallet(&self, reference: &WalletId) -> anyhow::Result<Wallet> {
        // TODO: File lock
        let store = self.load_unlocked().await?;
        Ok(store
            .wallets
            .get(&reference)
            .ok_or_else(|| anyhow!("Wallet reference not found: {:?}", &reference))?
            .to_owned())
    }

    pub async fn add_random_phrase(
        &self,
        _mnemonic_type: MnemonicType,
        _lang: Language,
    ) -> anyhow::Result<Wallet> {
        todo!()
    }

    pub async fn add_random_seed(&self, reference: WalletId) -> anyhow::Result<Wallet> {
        let wallet = Wallet::Seed(Seed::random());
        self.add(reference, wallet.clone()).await?;
        Ok(wallet)
    }

    pub async fn add_random_private(&self, reference: WalletId) -> anyhow::Result<Wallet> {
        let wallet = Wallet::Private(Private::random());
        self.add(reference, wallet.clone()).await?;
        Ok(wallet)
    }

    /// Add a new wallet to the store.
    ///
    /// If the wallet reference already exists, there will be an error.
    pub async fn add(&self, reference: WalletId, wallet: Wallet) -> anyhow::Result<()> {
        // TODO: File lock
        let mut storage = self.load_unlocked().await?;
        if storage.wallets.contains_key(&reference) {
            return Err(anyhow!("Wallet reference already exists: {:?}", &reference));
        }

        storage.wallets.insert(reference.clone(), wallet);
        let file = File::create(&self.path).await?;
        self.save_unlocked(file, storage).await?;
        Ok(())
    }
}

/// The secret of an individual wallet.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Wallet {
    /// A wallet that derives keys from a phrase.
    /// TODO: Change `Phrase` so that it can be Serialized
    // Phrase(Phrase),

    /// A wallet that derives from a seed.
    Seed(Seed),

    /// A wallet with a single private key.
    Private(Private),
}

impl Wallet {
    /// Derive or return a private key for this wallet.
    pub fn private(&self, index: u32) -> anyhow::Result<Private> {
        match &self {
            Wallet::Seed(seed) => Ok(seed.derive(index)),
            Wallet::Private(private) => {
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

/// Storage for all wallets.
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStorage {
    wallets: HashMap<WalletId, Wallet>,
}

impl WalletStorage {
    pub fn new() -> Self {
        Self {
            wallets: Default::default(),
        }
    }
}

/// A unique identifier for a wallet. This can be generated randomly and given to the user for
/// future reference, or given by the user.
#[derive(Hash, Eq, PartialEq, Clone)]
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

impl Serialize for WalletId {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for WalletId {
    /// Convert from a string of hex into a `WalletId` [u8; ..]
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let vec = hex::decode(s.as_bytes()).map_err(D::Error::custom)?;
        let decoded = vec.as_slice();
        let d = <[u8; WalletId::LEN]>::try_from(decoded).map_err(D::Error::custom)?;
        Ok(Self(d))
    }
}

impl Debug for WalletId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::fs::remove_file;
    use std::str::FromStr;

    /// Remove the wallet file when dropped.
    struct Clean(PathBuf);
    impl Drop for Clean {
        fn drop(&mut self) {
            remove_file(&self.0).unwrap()
        }
    }

    async fn prepare(p: &str) -> (Clean, WalletManager) {
        let p = PathBuf::from_str(p).unwrap();
        if p.exists() {
            remove_file(p.clone()).unwrap();
        }
        let manager = WalletManager::new(p.clone());
        manager.ensure().await.unwrap();
        (Clean(p), manager)
    }

    #[tokio::test]
    async fn sanity_check() {
        let (_clean, manager) = prepare("test.wallet").await;
        let w1 = manager.add_random_seed(WalletId::zero()).await.unwrap();
        let w2 = manager.wallet(&WalletId::zero()).await.unwrap();
        assert_eq!(w1.address(0).unwrap(), w2.address(0).unwrap())
    }

    #[tokio::test]
    async fn import_seed() {
        let (_clean, manager) = prepare("import_seed.wallet").await;
        let seed =
            Seed::from_str("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        let wallet = Wallet::Seed(seed);
        let reference = WalletId::zero();
        manager.add(reference, wallet).await.unwrap();
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
