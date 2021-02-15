use crate::{BlockHash, Public};

#[derive(Debug)]
pub struct ReceiveBlock {
    previous: BlockHash,
    source: Public,
}
