use crate::{Phrase, Private, Seed};
use fd_lock::{FdLock, FdLockGuard};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;

/// A reference to a wallet file. **Warning**: Wallet files are not locked (yet).
pub struct Wallet {
    path: PathBuf,
}

impl Wallet {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    /// This should be called to create the file if it doesn't exists.
    pub async fn ensure(&self) -> anyhow::Result<()> {
        if self.path.exists() {
            return Ok(());
        }

        let store = Store::new();
        let file = File::create(&self.path)?;
        serde_json::to_writer_pretty(file, &store)?;

        Ok(())
    }

    pub async fn load(&self) -> anyhow::Result<(File, Store)> {
        self.ensure().await?;

        let file = File::open(&self.path)?;
        let store: Store = serde_json::from_reader(&file)?;
        Ok((file, store))
    }

    pub async fn save(&self, file: File, store: Store) -> anyhow::Result<()> {
        Ok(serde_json::to_writer_pretty(file, &store)?)
    }
}

/// An individual wallet that can store a different type of seed, etc.
#[derive(Serialize, Deserialize)]
pub enum SingleWallet {
    /// A wallet that derives keys from a phrase.
    /// TODO: Change Phrase so that it can be Serialized
    // Phrase(Phrase),

    /// A wallet that derives from a seed.
    Seed(Seed),

    /// A wallet with a list of unrelated private keys.
    Keys(Vec<Private>),
}

/// Storage for all wallets.
#[derive(Serialize, Deserialize)]
pub struct Store {
    wallets: HashMap<WalletId, SingleWallet>,
}

impl Store {
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
        let wallet = Wallet::new("test.wallet");
        let (file, store) = wallet.load().await.unwrap();
        wallet.save(file, store).await.unwrap();
    }
}
