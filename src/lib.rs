#![forbid(unsafe_code)]

#[cfg(feature = "node")]
mod node;

#[cfg(feature = "pcap")]
mod pcap;

#[cfg(feature = "wallet")]
mod wallet;

pub mod blocks;
mod bytes;
pub mod cli;
mod debug;
mod encoding;
mod keys;
mod network;
mod pow;
mod raw;

use anyhow::anyhow;
pub(crate) use encoding::{hex_formatter, to_hex};
pub use keys::address::Address;
pub use keys::phrase;
pub use keys::phrase::Phrase;
pub use keys::private::Private;
pub use keys::public::Public;
pub use keys::seed::Seed;
pub use keys::signature::Signature;
pub use pow::work::Work;
pub use raw::Raw;

pub const DEFAULT_PORT: u16 = 7075;

fn expect_len(got_len: usize, expected_len: usize, msg: &str) -> anyhow::Result<()> {
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

fn len_err_msg(got_len: usize, expected_len: usize, msg: &str) -> String {
    format!(
        "{} is the wrong length: got: {} expected: {}",
        msg, got_len, expected_len,
    )
}
