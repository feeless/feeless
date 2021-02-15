use crate::{BlockHash, Public};

#[derive(Debug)]
pub struct ChangeBlock {
    previous: BlockHash,
    representative: Public,
}
