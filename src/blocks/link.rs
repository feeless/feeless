use crate::{expect_len, BlockHash, Public};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Link {
    /// For the change block type.
    Nothing,

    /// When we've received and decoded a block, but don't know what kind of block this is yet.
    Unsure([u8; Link::LEN]),

    /// Reference the previous block, for receiving.
    Source(BlockHash),

    /// Send to a destination account.
    DestinationAccount(Public),
}

impl Link {
    pub const LEN: usize = 32;

    pub fn nothing() -> Self {
        Self::Nothing
    }

    pub fn unsure_from_hex(s: &str) -> anyhow::Result<Self> {
        expect_len(s.len(), Self::LEN * 2, "Link")?;
        let mut slice = [0u8; Self::LEN];
        hex::decode_to_slice(s, &mut slice)?;
        Ok(Link::Unsure(slice))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Link::Nothing => &[0u8; Self::LEN],
            Link::Source(hash) => hash.as_bytes(),
            Link::DestinationAccount(key) => key.as_bytes(),
            Link::Unsure(b) => b.as_ref(),
        }
    }
}
