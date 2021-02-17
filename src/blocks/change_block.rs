use crate::{BlockHash, Public};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeBlock {
    previous: BlockHash,
    representative: Public,
}
