use crate::{Phrase, Private, Seed};
use fd_lock::{FdLock, FdLockGuard};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::ops::Deref;
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

    pub async fn load(&self) -> anyhow::Result<(File, WalletStorage)> {
        self.ensure().await?;

        let file = File::open(&self.path)?;
        let store: WalletStorage = serde_json::from_reader(&file)?;
        Ok((file, store))
    }

    pub async fn save(&self, file: File, store: WalletStorage) -> anyhow::Result<()> {
        Ok(serde_json::to_writer_pretty(file, &store)?)
    }
}

/// An individual wallet that can store a different type of seed, etc.
#[derive(Serialize, Deserialize)]
pub enum Wallet {
    /// A wallet that derives keys from a phrase.
    /// TODO: Change Phrase so that it can be Serialized
    // Phrase(Phrase),

    /// A wallet that derives from a seed.
    Seed(Seed),

    /// A wallet that only has a single private key.
    Private(Private),
}

/// Storage for all wallets.
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
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

    #[tokio::test]
    async fn simple() {
        // let wallet = WalletManager::new("test.wallet");
        // let (file, store) = wallet.load().await.unwrap();
        // wallet.save(file, store).await.unwrap();
    }

    #[tokio::test]
    async fn new_wallet_and_address() {
        // Just mulling over the API and CLI interaction...
        // feeless wallet create --file gak.wallet
        // FEELESS_WALLET=gak.wallet feeless wallet create
        // Should warn user if wallet exists:
        // * Warning: File already exists with (3) wallets.
        let manager = WalletManager::new("test.wallet");
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
        // let wallet = manager.new_phrase(Language::English, 24).await.unwrap();
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
        // let wallet = manager.new_key_set().await.unwrap();
        // let wallet = manager.wallet(wallet_id);
        // wallet is read only and not locked? no need to write to it once we have the seed.

        // let private = wallet.private(0).unwrap();
        // let private = wallet.public(0).unwrap();
        // let address = wallet.address(0).unwrap();
        // let signature = wallet.address(0).sign("hello").unwrap();

        // let wallet_id = wallet_file.new_seed().await.unwrap();
        // wallet.address(wallet_id)
    }
}
