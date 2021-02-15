use super::Block;
use crate::blocks::BlockType;
use crate::encoding::blake2b;
use crate::{expect_len, BlockHash, FullBlock, Public, Raw, Work};

#[derive(Debug)]
pub struct StateBlock {
    account: Public,
    previous: BlockHash,
    representative: Public,
    balance: Raw,
    link: Link,
}

impl StateBlock {
    pub const LEN: usize = 216;

    pub fn new(
        account: Public,
        previous: BlockHash,
        representative: Public,
        balance: Raw,
        link: Link,
    ) -> Self {
        Self {
            account,
            previous,
            representative,
            balance,
            link,
        }
    }

    pub fn into_full_block(self) -> FullBlock {
        FullBlock::new(Block::State(self))
    }

    pub fn hash(&self) -> anyhow::Result<BlockHash> {
        todo!()

        // TODO: use hash_block()

        // let mut v = Vec::new(); // TODO: with_capacity
        //
        // // Preamble: A u256 of the block type.
        // v.extend_from_slice(&[0u8; 31]);
        // v.push(BlockType::State as u8);
        //
        // v.extend_from_slice(self.account.as_bytes());
        // v.extend_from_slice(self.previous.as_bytes());
        // v.extend_from_slice(self.representative.as_bytes());
        // v.extend_from_slice(self.balance.to_vec().as_slice());
        // v.extend_from_slice(self.link.as_bytes());
        //
        // BlockHash::try_from(blake2b(BlockHash::LEN, &v).as_ref())
    }
}

// impl Wire for StateBlock {
//     fn serialize(&self) -> Vec<u8> {
//         unimplemented!()
//     }
//
//     fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
//         where
//             Self: Sized,
//     {
//         warn!("TODO StateBlock::deserialize");
//         let mut data = Bytes::new(data);
//
//         let account = Public::try_from(data.slice(Public::LEN)?)?;
//         let previous = BlockHash::try_from(data.slice(BlockHash::LEN)?)?;
//         let representative = Public::try_from(data.slice(Public::LEN)?)?;
//         let raw = Raw::try_from(data.slice(Raw::LEN)?)?;
//
//         let link_data = data.slice(Public::LEN)?;
//         // TODO: I think this only works once we have previous blocks in a database.
//         // let link_data_is_zero = link_data == [0u8; Public::LEN];
//         // let link = if diff < 0 {
//         //     // Send
//         //     info!("Senddddddddddddddd");
//         //     Link::SendDestinationPublicKey(Public::try_from(link_data)?)
//         // } else if raw > 0 {
//         //     // Receive
//         //     info!("Recvvvvvvvvvvvvvvv");
//         //     Link::PairingSendBlockHash(BlockHash::try_from(link_data)?)
//         // } else {
//         //     // Change rep
//         //     if !link_data_is_zero {
//         //         return Err(anyhow!("Link data is zero but raw is not zero: {:?}", raw));
//         //     }
//         //     info!("Changerepppppppppppppppp");
//         //     Link::Nothing
//         // };
//         let link = Link::Unsure(<[u8; 32]>::try_from(link_data)?);
//
//         let signature = Signature::try_from(data.slice(Signature::LEN)?)?;
//
//         Ok(Self::new(
//             account,
//             previous,
//             representative,
//             raw,
//             link,
//             signature,
//             Work::zero(), // TODO
//         ))
//     }
//
//     fn len(header: Option<&Header>) -> Result<usize, anyhow::Error> {
//         debug_assert!(header.is_some());
//         let header = header.unwrap();
//
//         if header.ext().block_type()? != BlockType::State {
//             return Err(anyhow!(
//                 "unexpected block type: {:?}",
//                 header.ext().block_type()
//             ));
//         }
//
//         Ok(VerifiableBlock::LEN)
//     }
// }

#[derive(Debug)]
pub enum Link {
    Nothing,
    Unsure([u8; Link::LEN]),
    PairingSendBlockHash(BlockHash),
    SendDestinationPublicKey(Public),
}

impl Link {
    pub const LEN: usize = 32;

    pub fn nothing() -> Self {
        Self::Nothing
    }

    pub fn unsure_from_hex(s: &str) -> anyhow::Result<Self> {
        expect_len(s.len(), Self::LEN * 2, "Link")?;
        let mut slice = [0u8; Self::LEN];
        hex::decode_to_slice(s, &mut slice)?;
        Ok(Link::Unsure(slice))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Link::Nothing => &[0u8; Self::LEN],
            Link::PairingSendBlockHash(hash) => hash.as_bytes(),
            Link::SendDestinationPublicKey(key) => key.as_bytes(),
            Link::Unsure(b) => b.as_ref(),
        }
    }
}
