use crate::{BlockHash, Public};

#[derive(Debug, Clone)]
pub struct ChangeBlock {
    previous: BlockHash,
    representative: Public,
}
