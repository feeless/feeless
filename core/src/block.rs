use crate::{Address, BlockHash, Public, Raw, Signature, Work};

// #[derive(Debug)]
// pub struct SendBlock {
//     previous: BlockHash,
//     destination: Public,
//     balance: Raw,
//     signature: Signature,
//     work: Work,
// }
//
// #[derive(Debug)]
// pub struct ReceiveBlock {
//     previous: BlockHash,
//     source: Public,
//     signature: Signature,
//     work: Work,
// }
//
// #[derive(Debug)]
// pub struct OpenBlock {
//     source: Address,
//     representative: Public,
//     account: Public,
//     signature: Signature,
//     work: Work,
// }
//
// #[derive(Debug)]
// pub struct ChangeBlock {
//     previous: BlockHash,
//     representative: Public,
//     signature: Signature,
//     work: Work,
// }

#[derive(Debug)]
pub struct StateBlock {
    account: Public,
    previous: BlockHash,
    representative: Public,
    balance: Raw,
    link: Public,
    signature: Signature,
    work: Work,
}

impl StateBlock {
    pub const LEN: usize = 216;

    pub fn new(
        account: Public,
        previous: BlockHash,
        representative: Public,
        balance: Raw,
        link: Public,
        signature: Signature,
        work: Work,
    ) -> Self {
        Self {
            account,
            previous,
            representative,
            balance,
            link,
            signature,
            work,
        }
    }
}
