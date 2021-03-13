#[cfg(feature = "node")]
use crate::node::Header;

#[cfg(feature = "node")]
use crate::node::Wire;

use crate::blocks::BlockHash;
use crate::Public;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeBlock {
    previous: BlockHash,
    representative: Public,
}
