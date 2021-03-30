//! File storage for seeds and private keys.
//!
//! # Manager
//! A [WalletManager] is provided to store multiple [Wallet]s of different types. The supported
//! wallets are [Wallet::Seed], [Wallet::Private], and (TODO) [Wallet::Phrase].
//!
//! ## Example usage
//! ```
//! use feeless::wallet::WalletManager;
//! use feeless::wallet::WalletId;
//! # use std::fs::remove_file;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let manager = WalletManager::new("my.wallet");
//! // Create if the file doesn't exist.
//! manager.ensure().await?;
//!
//! // Create a new wallet with a random seed.
//! let wallet_id = WalletId::random();
//! let wallet = manager.add_random_seed(wallet_id.to_owned()).await?;
//!
//! // Use the 3rd Nano address.
//! let address = wallet.address(2)?;
//!
//! // Grab an existing wallet
//! let wallet = manager.wallet(&wallet_id).await?;
//!
//! # remove_file("my.wallet")?;
//!
//! # Ok(())
//! # }
//! ```
use crate::phrase::{Language, MnemonicType};
use crate::{to_hex, Address, Phrase, Private, Public, Seed};
use anyhow::{anyhow, Context};
use rand::RngCore;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::File;
use crate::FeelessError;

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
        let file = File::open(&self.path)
            .await
            .with_context(|| format!("Opening {:?}", &self.path))?;
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
        id: WalletId,
        mnemonic_type: MnemonicType,
        lang: Language,
    ) -> anyhow::Result<Wallet> {
        let wallet = Wallet::Phrase(Phrase::random(mnemonic_type, lang));
        self.add(id, wallet.clone()).await?;
        Ok(wallet)
    }

    pub async fn add_random_seed(&self, id: WalletId) -> anyhow::Result<Wallet> {
        let wallet = Wallet::Seed(Seed::random());
        self.add(id, wallet.clone()).await?;
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
        let file = File::create(&self.path)
            .await
            .with_context(|| format!("Creating file {:?}", &self.path))?;
        self.save_unlocked(file, storage).await?;
        Ok(())
    }

    /// If the wallet reference doesn't exist, there will be an error.
    pub async fn delete(&self, reference: &WalletId) -> anyhow::Result<()> {
        let mut storage = self.load_unlocked().await?;
        if !storage.wallets.contains_key(reference) {
            return Err(anyhow!("Wallet reference doesn't exist: {:?}", &reference));
        }
        storage.wallets.remove(reference);
        let file = File::create(&self.path)
            .await
            .with_context(|| format!("Creating file {:?}", &self.path))?;
        self.save_unlocked(file, storage).await?;
        Ok(())
    }
}

/// The secret of an individual wallet.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Wallet {
    /// A wallet that derives keys from a phrase.
    Phrase(Phrase),

    /// A wallet that derives from a seed.
    Seed(Seed),

    /// A wallet with a single private key.
    Private(Private),
}

impl Wallet {
    /// Derive or return a private key for this wallet.
    pub fn private(&self, index: u32) -> Result<Private, FeelessError> {
        match &self {
            Wallet::Seed(seed) => Ok(seed.derive(index)),
            Wallet::Private(private) => {
                if index != 0 {
                   return Err(FeelessError::WalletError);
                }
                Ok(private.to_owned())
            }
            Wallet::Phrase(phrase) => Ok(phrase.to_private(index, "")?),
        }
    }

    /// Derive a public key for this wallet.
    pub fn public(&self, index: u32) -> Result<Public, FeelessError> {
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

    pub(crate) fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn random() -> Self {
        let mut id = Self::zero();
        rand::thread_rng().fill_bytes(&mut id.0);
        id
    }
}

impl FromStr for WalletId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = hex::decode(s.as_bytes())?;
        let decoded = vec.as_slice();
        let d = <[u8; WalletId::LEN]>::try_from(decoded)?;
        Ok(Self(d))
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
        Self::from_str(s.as_str()).map_err(D::Error::custom)
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
        manager.add(reference, wallet.to_owned()).await.unwrap();

        assert_eq!(
            wallet.address(0).unwrap(),
            Address::from_str("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7")
                .unwrap()
        );
    }
}