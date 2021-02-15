use crate::{BlockHash, Public};

#[derive(Debug, Clone)]
pub struct ReceiveBlock {
    previous: BlockHash,
    source: Public,
}
