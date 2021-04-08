use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("From hex error: {msg} {source}")]
    FromHexError {
        msg: String,
        source: hex::FromHexError,
    },

    #[error("Signature error: {msg} {source}")]
    SignatureError {
        msg: String,
        source: ed25519_dalek::SignatureError,
    },

    #[error("Try from slice error")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    #[error("There is only one private key in this wallet. Only use index 0.")]
    WalletError,

    #[error("Invalid Nano address")]
    InvalidAddress,

    #[error("Unknown character found while decoding: {0}")]
    DecodingError(char),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Bad public key, can not verify")]
    BadPublicKey,

    #[error("Extended secret key error")]
    ExtendedSecretKeyError(#[from] ed25519_dalek_bip32::Error),

    #[error("Mnemonic error")]
    MnemonicError(#[from] bip39::ErrorKind),

    #[error("Wrong length for {msg} (expected {expected:?}, found {found:?})")]
    WrongLength {
        msg: String,
        expected: usize,
        found: usize,
    },

    #[error("Parse int error")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Parse big decimal error")]
    ParseBigDecimalError(#[from] bigdecimal::ParseBigDecimalError),

    #[error("Possible language codes are {0}")]
    LanguageError(String),

    #[error("Invalid armor content: {0}")]
    InvalidArmor(String),

    #[error("RPC request failed: {0}")]
    RPCRequestFailed(#[from] reqwest::Error),

    #[error("Bad RPC response: {err:?} response: {response}")]
    BadRPCResponse {
        err: serde_json::Error,
        response: String,
    },

    #[error("RPC error: {0}")]
    RPCError(String),

    #[error("IO error: {msg} {source}")]
    IOError { msg: String, source: std::io::Error },

    #[error("The file doesn't exist")]
    NonExistentFile,

    #[error("You haven't defined a password")]
    UndefinedPassword,

    #[error("Wallet id error")]
    WalletIdError(String),

    #[error("Error reading file")]
    ReadError(#[from] serde_json::Error),

    #[error("Error decrypting file")]
    DecryptionError(#[from] age::DecryptError),
    
    //#[error("Error encrypting file")]
    //EncryptionError(#[from] age::EncryptError),
    // TODO: this is giving an error, but I don't understand why
}
