use header::Header;
use std::fmt::Debug;

pub mod cookie;
pub mod header;
pub mod peer;
pub mod raw;
pub mod state_block;

pub trait Wire: Debug {
    fn serialize(&self) -> Vec<u8>;

    /// Only when deserializing the header we don't have a header.
    /// This should .expect() when unwrapping when it's not a header, since it'll be a
    /// programming error.
    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// The expected size of the incoming data.
    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized;
}
