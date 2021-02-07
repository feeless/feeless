use crate::header::Header;
use crate::state::SledState;

pub trait Wire {
    fn serialize(&self) -> Vec<u8>;

    /// Only when deserializing the header we don't have a header.
    /// This should .expect() when unwrapping when it's not a header, since it'll be a
    /// programming error.
    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// The expected size of the incoming data.
    fn len(header: Option<&Header>) -> usize;
}
