use crate::bytes::Bytes;
use crate::header::{BlockType, Header};
use crate::wire::Wire;
use anyhow::anyhow;
use feeless::{BlockHash, Link, Public, Raw, Signature, StateBlock, Work};
use std::convert::TryFrom;
use tracing::{info, warn};

/// A wrapper around StateBlock with serialization.
#[derive(Debug)]
pub struct WireStateBlock(StateBlock);

impl Wire for WireStateBlock {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        warn!("TODO Wire::deserialize");
        let mut data = Bytes::new(data);

        let account = Public::try_from(data.slice(Public::LEN)?)?;
        let previous = BlockHash::try_from(data.slice(BlockHash::LEN)?)?;
        let representative = Public::try_from(data.slice(Public::LEN)?)?;
        let raw = Raw::try_from(data.slice(Raw::LEN)?)?;

        let link_data = data.slice(Public::LEN)?;
        // TODO: I think this only works once we have previous blocks in a database.
        // let link_data_is_zero = link_data == [0u8; Public::LEN];
        // let link = if diff < 0 {
        //     // Send
        //     info!("Senddddddddddddddd");
        //     Link::SendDestinationPublicKey(Public::try_from(link_data)?)
        // } else if raw > 0 {
        //     // Receive
        //     info!("Recvvvvvvvvvvvvvvv");
        //     Link::PairingSendBlockHash(BlockHash::try_from(link_data)?)
        // } else {
        //     // Change rep
        //     if !link_data_is_zero {
        //         return Err(anyhow!("link data is zero but raw is not zero: {:?}", raw));
        //     }
        //     info!("Changerepppppppppppppppp");
        //     Link::Nothing
        // };
        let link = Link::Nothing;

        let signature = Signature::try_from(data.slice(Signature::LEN)?)?;

        Ok(Self(StateBlock::new(
            account,
            previous,
            representative,
            raw,
            link,
            signature,
            Work::zero(), // TODO
        )))
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error> {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type()? != BlockType::State {
            return Err(anyhow!(
                "Unexpected block type: {:?}",
                header.ext().block_type()
            ));
        }

        Ok(StateBlock::LEN)
    }
}
