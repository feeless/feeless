#![allow(dead_code)]
#![forbid(unsafe_code)]
#![cfg_attr(feature = "deny_warnings", deny(warnings))]
// #![warn(missing_docs)] LOL not yet.
//! A set of tools to handle many aspects of the Nano cryptocurrency.
//!
//! See the [feeless.dev website](https://feeless.dev) for more details about this project.
//!
//! ## Keys and signing example
//! ```
//! use feeless::Phrase;
//! use feeless::phrase::Language;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Example phrase from https://docs.nano.org/integration-guides/key-management/#test-vectors
//! let words = "edge defense waste choose enrich upon flee junk siren film clown finish
//!              luggage leader kid quick brick print evidence swap drill paddle truly occur";
//!
//! // Generate the private key from the seed phrase.
//! let phrase = Phrase::from_words(Language::English, words)?;
//!
//! // First account with the password `some password`.
//! let private_key = phrase.to_private(0, "some password")?;
//! let public_key = private_key.to_public()?;
//! let address = public_key.to_address();
//!
//! // The above three lines can be chained like this:
//! let address = phrase.to_private(0, "some password")?.to_public()?.to_address();
//! assert_eq!(address.to_string(), "nano_1pu7p5n3ghq1i1p4rhmek41f5add1uh34xpb94nkbxe8g4a6x1p69emk8y1d");
//!     
//! // Sign a message.
//! let message = "secret message!".as_bytes();
//! let signature = private_key.sign(message)?;
//!
//! // Someone else can verify the message based on your address or public key.
//! address.to_public().verify(message, &signature)?;
//! public_key.verify(message, &signature)?;
//!     
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "node")]
mod node;

#[cfg(feature = "pcap")]
mod pcap;

#[cfg(feature = "wallet")]
pub mod wallet;

#[doc(hidden)]
pub mod cli;

pub mod blocks;
mod bytes;
mod debug;
mod encoding;
mod errors;
mod keys;
mod network;
mod pow;
pub mod rpc;
pub mod units;
pub mod vanity;
mod version;

pub use errors::{Error, Result};
pub use keys::address::Address;
pub use keys::phrase;
pub use keys::phrase::Phrase;
pub use keys::private::Private;
pub use keys::public::Public;
pub use keys::seed::Seed;
pub use keys::signature::Signature;
pub use network::{Network, DEFAULT_PORT};
pub use pow::{Difficulty, Subject, Work};
pub use units::rai::Rai;
pub use version::Version;
