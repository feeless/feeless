use super::Block;
use crate::blocks::{hash_block, BlockType};
use crate::encoding::blake2b;
use crate::{expect_len, BlockHash, FullBlock, Public, Raw, Work};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
        let mut preamble = [0u8; 32];
        preamble[31] = BlockType::State as u8;

        hash_block(&[
            &preamble,
            self.account.as_bytes(),
            self.previous.as_bytes(),
            self.representative.as_bytes(),
            self.balance.to_vec().as_slice(),
            self.link.as_bytes(),
        ])
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::{Block, FullBlock, StateBlock};
    use super::{BlockHash, Raw};
    use crate::encoding::FromHex;
    use crate::{Address, Link, Signature, Work};
    use std::convert::TryFrom;

    #[test]
    fn hash_a_real_state_block() {
        let account =
            Address::from_str("nano_34prihdxwz3u4ps8qjnn14p7ujyewkoxkwyxm3u665it8rg5rdqw84qrypzk")
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

        let mut block =
            StateBlock::new(account, parent, representative, balance, link).into_full_block();
        block.set_signature(signature).unwrap();
        block.set_work(work).unwrap();

        assert_eq!(
            block.hash().unwrap(),
            BlockHash::from_hex("6F050D3D0B19C2C206046AAE2D46661B57E1B7D890DE8398D203A025E29A4AD9")
                .unwrap()
        )
    }
}
