use crate::keys::public::{from_address, to_address};
use crate::raw::{deserialize_from_hex, serialize_to_hex};
use crate::{BlockHash, Public, Raw, Signature, Work};
use serde::{Deserialize, Serialize};

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
