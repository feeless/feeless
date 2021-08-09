use std::fmt::Debug;

use crate::transport::header::Header;

pub trait Wire: Debug {
    fn serialize(&self) -> Vec<u8>;

    /// `header` will be `None` when we're deserializing the header itself.
    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn deserialize_payload(header: &Header, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Self::deserialize(Some(header), data)
    }

    /// The expected size of the incoming data.
    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized;

    fn len_given_header(header: &Header) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        Self::len(Some(header))
    }

    /// The expected size of the incoming data.
    fn len_payload(header: &Header) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        Self::len(Some(header))
    }
}
