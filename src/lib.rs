#![forbid(unsafe_code)]

#[cfg(feature = "node")]
pub mod node;

#[cfg(feature = "pcap")]
pub mod pcap;

use anyhow::anyhow;
pub use blocks::Block;
pub use blocks::BlockHash;
pub use blocks::Previous;
pub use encoding::{hex_formatter, to_hex};
pub use keys::address::Address;
pub use keys::phrase::{Language, MnemonicType, Phrase};
pub use keys::private::Private;
pub use keys::public::Public;
pub use keys::seed::Seed;
pub use keys::signature::Signature;
pub use pow::work::Work;
pub use raw::Raw;

pub mod blocks;
mod bytes;
pub mod debug;
pub mod encoding;
mod keys;
pub mod network;
mod pow;
mod raw;

pub const DEFAULT_PORT: u16 = 7075;

pub fn expect_len(got_len: usize, expected_len: usize, msg: &str) -> anyhow::Result<()> {
    if got_len != expected_len {
        return Err(anyhow!(
            "{} is the wrong length: got: {} expected: {}",
            msg,
            got_len,
            expected_len,
        ));
    }
    Ok(())
}

pub fn len_err_msg(got_len: usize, expected_len: usize, msg: &str) -> String {
    format!(
        "{} is the wrong length: got: {} expected: {}",
        msg, got_len, expected_len,
    )
}
