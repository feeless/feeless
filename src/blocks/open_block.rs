use crate::blocks::BlockHash;
use crate::keys::public::{from_address, to_address};
use crate::{Public, Signature, Work};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenBlock {
    pub source: BlockHash,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub representative: Public,

    #[serde(serialize_with = "to_address", deserialize_with = "from_address")]
    pub account: Public,

    pub work: Option<Work>,
    pub signature: Option<Signature>,
}

impl OpenBlock {
    pub fn new(source: BlockHash, representative: Public, account: Public) -> Self {
        Self {
            source,
            representative,
            account,
            work: None,
            signature: None,
        }
    }
}
