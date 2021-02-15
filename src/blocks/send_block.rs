use crate::{BlockHash, Public, Raw};

#[derive(Debug, Clone)]
pub struct SendBlock {
    previous: BlockHash,
    destination: Public,
    balance: Raw,
}

impl SendBlock {}
