#[cfg(feature = "node")]
use crate::node::Header;

#[cfg(feature = "node")]
use crate::node::Wire;

use crate::blocks::{BlockHash, BlockType};
use crate::bytes::Bytes;
use crate::keys::public::{from_address, to_address};
use crate::raw::{deserialize_from_hex, serialize_to_hex};
use crate::{Public, Raw, Signature, Work};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SendBlock {
    /// The hash of the previous block in this account.
    pub previous: BlockHash,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub destination: Public,

    #[serde(
        serialize_with = "serialize_to_hex",
        deserialize_with = "deserialize_from_hex"
    )]
    pub balance: Raw,

    pub work: Option<Work>,
    pub signature: Option<Signature>,
}

impl SendBlock {
    pub const LEN: usize = 152;

    pub fn new(previous: BlockHash, destination: Public, balance: Raw) -> Self {
        Self {
            previous,
            destination,
            balance,
            work: None,
            signature: None,
        }
    }
}

#[cfg(feature = "node")]
impl Wire for SendBlock {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut data = Bytes::new(data);
        let previous = BlockHash::try_from(data.slice(BlockHash::LEN)?)?;
        let destination = Public::try_from(data.slice(Public::LEN)?)?;
        let balance = Raw::try_from(data.slice(Raw::LEN)?)?;
        let work = Some(Work::try_from(data.slice(Work::LEN)?)?);
        let signature = Some(Signature::try_from(data.slice(Signature::LEN)?)?);

        Ok(Self {
            previous,
            destination,
            balance,
            work,
            signature,
        })
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        debug_assert!(header.is_some());
        let header = header.unwrap();
        debug_assert_eq!(header.ext().block_type()?, BlockType::Send);

        Ok(SendBlock::LEN)
    }
}
