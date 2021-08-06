use crate::blocks::BlockHash;
use crate::{Public, Signature, Work};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiveBlock {
    previous: BlockHash,
    source: Public,
    pub work: Option<Work>,
    pub signature: Option<Signature>,
}

impl ReceiveBlock {
    pub const LEN: usize = 32+32+64+8;
}