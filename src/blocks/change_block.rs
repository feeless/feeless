use crate::blocks::BlockHash;
use crate::{Public, Signature, Work};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeBlock {
    previous: BlockHash,
    representative: Public,
    pub work: Option<Work>,
    pub signature: Option<Signature>,
}

impl ChangeBlock {
    pub const LEN: usize = 32+32+64+8;
}