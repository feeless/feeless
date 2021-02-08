use crate::encoding::blake2b;
use crate::{Address, BlockHash, Public, Raw, Signature, Work};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct SendBlock {
    previous: BlockHash,
    destination: Public,
    balance: Raw,
    signature: Signature,
    work: Work,
}

#[derive(Debug)]
pub struct ReceiveBlock {
    previous: BlockHash,
    source: Public,
    signature: Signature,
    work: Work,
}

#[derive(Debug)]
pub struct OpenBlock {
    source: Address,
    representative: Public,
    account: Public,
    signature: Signature,
    work: Work,
}

#[derive(Debug)]
pub struct ChangeBlock {
    previous: BlockHash,
    representative: Public,
    signature: Signature,
    work: Work,
}

#[derive(Debug)]
pub struct StateBlock {
    account: Public,
    previous: BlockHash,
    representative: Public,
    balance: Raw,
    link: Link,
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
        link: Link,
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

    fn hash(&self) -> anyhow::Result<BlockHash> {
        let mut v = Vec::new(); // TODO: with_capacity
        v.extend_from_slice(self.account.as_bytes());
        v.extend_from_slice(self.previous.as_bytes());
        Ok(BlockHash::try_from(blake2b(BlockHash::LEN, &v).as_ref())?)
    }
}

#[derive(Debug)]
pub enum Link {
    Nothing,
    PairingSendBlockHash(BlockHash),
    SendDestinationPublicKey(Public),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn hash() {
        let account =
            Address::try_from("nano_3x4ui45q1cw8hydmfdn4ec5ijsdqi4ryp14g4ayh71jcdkwmddrq7ca9xzn9")
                .unwrap()
                .to_public();
        let parent =
            BlockHash::from_hex("2656EAE462DD2D77B9405E5BA3822D54EDCFFD4D88238BB54FA786A22C7B07F8")
                .unwrap();
        let representative = account.clone();
        let balance = Raw::from_mnano(123u128);
        let link = Link::Nothing;
        let signature = Signature::zero();
        let work = Work::zero();

        let b = StateBlock::new(
            account,
            parent,
            representative,
            balance,
            link,
            signature,
            work,
        );

        assert_eq!(
            b.hash().unwrap(),
            BlockHash::from_hex("41FFA433A23AC4EA10CFF3608AB703C79726252D6B787C907DC9CA04580EBF9A")
                .unwrap()
        )
    }
}
