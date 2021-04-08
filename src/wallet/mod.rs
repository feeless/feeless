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
//! let wallet = manager.wallet(&wallet_id, None).await?;
//!
//! # remove_file("my.wallet")?;
//!
//! # Ok(())
//! # }
//! ```
use crate::phrase::{Language, MnemonicType};
use crate::{to_hex, Address, Phrase, Private, Public, Seed};
use crate::{Error, Result};
use rand::RngCore;
use secrecy::Secret;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::File;
use tokio::fs::{read, write};

const DEFAULT_WALLET_FILE: &str = "default.wallet";

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

    pub fn default_wallet_file(file: &Option<PathBuf>) -> WalletManager {
        let mut manager = WalletManager::new(DEFAULT_WALLET_FILE);
        if let Some(f) = file {
            manager = WalletManager::new(f);
        }
        manager
    }

    /// This should be called to create the file if it doesn't exist.
    pub async fn ensure(&self) -> Result<()> {
        if self.path.exists() {
            return Ok(());
        }

        let store = WalletStorage::new();
        let file = File::create(&self.path).await.map_err(|e| Error::IOError {
            msg: format!("Opening file {:?}", &self.path),
            source: e,
        })?;
        serde_json::to_writer_pretty(file.into_std().await, &store)?;

        Ok(())
    }

    /// An internal method for loading the wallet storage.
    ///
    /// TODO: There should be a file lock around this.
    pub(crate) async fn load_unlocked(&self) -> Result<WalletStorage> {
        let file = File::open(&self.path).await.map_err(|e| Error::IOError {
            msg: format!("Opening file {:?}", &self.path),
            source: e,
        })?;
        let store: WalletStorage = serde_json::from_reader(&file.into_std().await)?;
        Ok(store)
    }

    /// An internal method for save the wallet storage.
    ///
    /// TODO: There should be a file lock around this.
    async fn save_unlocked(&self, file: File, store: WalletStorage) -> Result<()> {
        Ok(serde_json::to_writer_pretty(file.into_std().await, &store)?)
    }

    pub async fn wallet(&self, reference: &WalletId, password: Option<&str>) -> Result<Wallet> {
        // TODO: File lock
        match password {
            None => {
                let store = self.load_unlocked().await?;
                Ok(store
                    .wallets
                    .get(&reference)
                    .ok_or_else(|| {
                        Error::WalletIdError(format!("Wallet id {:?} doesn't exist", &reference))
                    })?
                    .to_owned())
            }
            Some(password) => {
                let decrypted = self.decrypt(&password, true).await?;
                let wallet_storage: std::result::Result<WalletStorage, serde_json::error::Error> =
                    serde_json::from_slice(&decrypted);
                match wallet_storage {
                    Ok(store) => Ok(store
                        .wallets
                        .get(&reference)
                        .ok_or_else(|| {
                            Error::WalletIdError(format!(
                                "Wallet id {:?} doesn't exist",
                                &reference
                            ))
                        })?
                        .to_owned()),
                    Err(_) => Err(Error::UndefinedPassword),
                }
            }
        }
    }

    pub async fn add_random_phrase(
        &self,
        id: WalletId,
        mnemonic_type: MnemonicType,
        lang: Language,
    ) -> Result<Wallet> {
        let wallet = Wallet::Phrase(Phrase::random(mnemonic_type, lang));
        self.add(id, wallet.clone()).await?;
        Ok(wallet)
    }

    pub async fn add_random_seed(&self, id: WalletId) -> Result<Wallet> {
        let wallet = Wallet::Seed(Seed::random());
        self.add(id, wallet.clone()).await?;
        Ok(wallet)
    }

    pub async fn add_random_private(&self, reference: WalletId) -> Result<Wallet> {
        let wallet = Wallet::Private(Private::random());
        self.add(reference, wallet.clone()).await?;
        Ok(wallet)
    }

    /// Add a new wallet to the store.
    ///
    /// If the wallet reference already exists, there will be an error.
    pub async fn add(&self, reference: WalletId, wallet: Wallet) -> Result<()> {
        // TODO: File lock
        let mut storage = self.load_unlocked().await?;
        if storage.wallets.contains_key(&reference) {
            return Err(Error::WalletIdError(format!(
                "Wallet id {:?} already exists",
                &reference
            )));
        }

        storage.wallets.insert(reference.clone(), wallet);
        let file = File::create(&self.path).await.map_err(|e| Error::IOError {
            msg: format!("Creating file {:?}", &self.path),
            source: e,
        })?;
        self.save_unlocked(file, storage).await?;
        Ok(())
    }

    /// Encrypt the wallet file with a password.
    pub async fn encrypt(&self, password: &str) -> Result<()> {
        let file = read(&self.path).await.map_err(|e| Error::IOError {
            msg: format!("Reading file {:?}", &self.path),
            source: e,
        })?;
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(password.to_owned()));
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted).unwrap(); // TODO: see errors.rs
        writer.write_all(&file).map_err(|e| Error::IOError {
            msg: format!("Writing file {:?}", &self.path),
            source: e,
        })?;
        writer.finish().map_err(|e| Error::IOError {
            msg: format!("Writing file {:?}", &self.path),
            source: e,
        })?;
        write(&self.path, &encrypted)
            .await
            .map_err(|e| Error::IOError {
                msg: format!("Writing file {:?}", &self.path),
                source: e,
            })?;
        Ok(())
    }

    /// Decrypt the wallet file.
    pub async fn decrypt(&self, password: &str, only_read: bool) -> Result<Vec<u8>> {
        let file = read(&self.path).await.map_err(|e| Error::IOError {
            msg: format!("Reading file {:?}", &self.path),
            source: e,
        })?;
        let decrypted = {
            let decryptor = match age::Decryptor::new(file.as_slice())? {
                age::Decryptor::Passphrase(d) => d,
                // TODO: find out why it is unreachable
                _ => unreachable!(),
            };

            let mut decrypted = vec![];
            let mut reader = decryptor.decrypt(&Secret::new(password.to_owned()), None)?;
            reader
                .read_to_end(&mut decrypted)
                .map_err(|e| Error::IOError {
                    msg: format!("Reading file {:?}", &self.path),
                    source: e,
                })?;

            decrypted
        };
        if !only_read {
            write(&self.path, decrypted.clone())
                .await
                .map_err(|e| Error::IOError {
                    msg: format!("Writing file {:?}", &self.path),
                    source: e,
                })?;
        }
        Ok(decrypted)
    }

    /// If the wallet reference doesn't exist, there will be an error.
    pub async fn delete(&self, reference: &WalletId) -> Result<()> {
        let mut storage = self.load_unlocked().await?;
        if !storage.wallets.contains_key(reference) {
            return Err(Error::WalletIdError(format!(
                "Wallet id {:?} doesn't exist",
                &reference
            )));
        }
        storage.wallets.remove(reference);
        let file = File::create(&self.path).await.map_err(|e| Error::IOError {
            msg: format!("Creating file {:?}", &self.path),
            source: e,
        })?;
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
    pub fn private(&self, index: u32) -> Result<Private> {
        match &self {
            Wallet::Seed(seed) => Ok(seed.derive(index)),
            Wallet::Private(private) => {
                if index != 0 {
                    return Err(Error::WalletError);
                }
                Ok(private.to_owned())
            }
            Wallet::Phrase(phrase) => Ok(phrase.to_private(index, "")?),
        }
    }

    /// Derive a public key for this wallet.
    pub fn public(&self, index: u32) -> Result<Public> {
        self.private(index)?.to_public()
    }

    /// Derive an address for this wallet.
    pub fn address(&self, index: u32) -> Result<Address> {
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let vec = hex::decode(s.as_bytes()).map_err(|e| Error::FromHexError {
            msg: String::from("Decoding hex wallet id"),
            source: e,
        })?;
        let decoded = vec.as_slice();
        let d = <[u8; WalletId::LEN]>::try_from(decoded)?;
        Ok(Self(d))
    }
}

impl Serialize for WalletId {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> core::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for WalletId {
    /// Convert from a string of hex into a `WalletId` [u8; ..]
    fn deserialize<D>(
        deserializer: D,
    ) -> core::result::Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(s.as_str()).map_err(serde::de::Error::custom)
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
        let w2 = manager.wallet(&WalletId::zero(), None).await.unwrap();
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
