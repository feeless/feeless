use thiserror::Error;

#[derive(Debug, Error)]
pub enum FeelessError {
    #[error("from hex error")]
    FromHexError(#[from] hex::FromHexError), 
    #[error("signature error")]
    SignatureError(#[from] ed25519_dalek::SignatureError), 
    #[error("try from slice error")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("there is only one private key in this wallet. Only use index 0.")]
    WalletError,
    #[error("invalid nano address")]
    InvalidAddress,
    #[error("unknown character found while decoding: {0}")]
    DecodingError(char),
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("bad public key, can not verify")]
    BadPublicKey,
    #[error("extended secret key error")]
    ExtendedSecretKeyError(#[from] ed25519_dalek_bip32::Error),
    #[error("mnemonic error")]
    MnemonicError(#[from] bip39::ErrorKind),
    #[error("wrong length (expected {expected:?}, found {found:?})")]
    WrongLength {
        expected: usize,
        found: usize,
    }, 
    #[error("parse int error")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("parse big decimal error")]
    ParseBigDecimalError(#[from] bigdecimal::ParseBigDecimalError), 
    #[error("possible language codes are {0}")]
    LanguageError(String,)
} 