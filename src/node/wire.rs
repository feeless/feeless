use std::fmt::Debug;

use crate::node::header::Header;

pub trait Wire: Debug {
    fn serialize(&self) -> Vec<u8>;

    /// `header` will be `None` when we're deserializing the header itself.
    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// The expected size of the incoming data.
    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized;
}
