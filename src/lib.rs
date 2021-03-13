#![forbid(unsafe_code)]
// #![warn(missing_docs)] LOL not yet.
//! A set of tools to handle many aspects of the Nano cryptocurrency.
//!
//! feeless can be used as:
//! * A library. (This crate!)
//! * A command line tool with piping capability. See [examples/cli.rs](https://github.com/feeless/feeless/blob/main/examples/cli.rs) for example usage.
//! * A node. ⚠ WIP. Not a proper node yet, but lots of groundwork so far!

#[cfg(feature = "node")]
mod node;

#[cfg(feature = "pcap")]
mod pcap;

#[cfg(feature = "wallet")]
mod wallet;

#[doc(hidden)]
pub mod cli;

pub mod blocks;
mod bytes;
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

/// The default TCP port that Nano nodes use.
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
