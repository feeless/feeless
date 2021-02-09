use crate::encoding::blake2b;
use crate::{to_hex, Address, BlockHash, Public, Raw, Signature, Work};
use std::convert::TryFrom;

pub enum BlockType {
    Invalid = 0,
    NotABlock = 1,
    Send = 2,
    Receive = 3,
    Open = 4,
    Change = 5,
    State = 6,
}

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

        // Preamble: A u256 of the block type.
        v.extend_from_slice(&[0u8; 31]);
        v.push(BlockType::State as u8);

        v.extend_from_slice(self.account.as_bytes());
        v.extend_from_slice(self.previous.as_bytes());
        v.extend_from_slice(self.representative.as_bytes());
        v.extend_from_slice(self.balance.to_vec().as_slice());
        v.extend_from_slice(self.link.as_bytes());

        BlockHash::try_from(blake2b(BlockHash::LEN, &v).as_ref())
    }
}

#[derive(Debug)]
pub enum Link {
    Nothing,
    PairingSendBlockHash(BlockHash),
    SendDestinationPublicKey(Public),
}

impl Link {
    pub const LEN: usize = 32;

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Link::Nothing => &[0u8; Self::LEN],
            Link::PairingSendBlockHash(hash) => hash.as_bytes(),
            Link::SendDestinationPublicKey(key) => key.as_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn hash() {
        let account =
            Address::try_from("nano_34prihdxwz3u4ps8qjnn14p7ujyewkoxkwyxm3u665it8rg5rdqw84qrypzk")
                .unwrap()
                .to_public();
        let parent =
            BlockHash::from_hex("7837C80964CAD551DEABE162C7FC4BB58688A0C6EB6D9907C0D2A7C74A33C7EB")
                .unwrap();
        let representative = account.clone();
        let balance = Raw::from_raw(2711469892748129430069222848295u128);
        let link = Link::PairingSendBlockHash(
            BlockHash::from_hex("0399B19B022D260F3DDFBA26D0306D423F1890D3AE06136FAB16802D1F2B87A7")
                .unwrap(),
        );
        // Signature and work aren't hashed, but left them as the real data anyway.
        let signature = Signature::from_hex("BCF9F123138355AE9E741912D319FF48E5FCCA39D9E5DD74411D32C69B1C7501A0BF001C45D4F68CB561B902A42711E6166B9018E76C50CC868EF2E32B78F200").unwrap();
        let work = Work::from_hex("d4757052401b9e08").unwrap();

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
            BlockHash::from_hex("6F050D3D0B19C2C206046AAE2D46661B57E1B7D890DE8398D203A025E29A4AD9")
                .unwrap()
        )
    }
}
