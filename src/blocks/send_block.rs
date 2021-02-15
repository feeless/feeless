use crate::{BlockHash, Public, Raw};

#[derive(Debug)]
pub struct SendBlock {
    previous: BlockHash,
    destination: Public,
    balance: Raw,
}

impl SendBlock {}
