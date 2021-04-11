use crate::hexify;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BlockHash([u8; BlockHash::LEN]);

hexify!(BlockHash, "block hash");

impl BlockHash {
    pub const LEN: usize = 32;

    pub fn zero() -> Self {
        Self([0u8; BlockHash::LEN])
    }
}
