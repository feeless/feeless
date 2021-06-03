use crate::blocks::{BlockHash, BlockHolder, Subtype};
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::{Address, Raw, Result};
use async_trait::async_trait;
use chrono::Utc;
use clap::Clap;
use serde::{Deserialize, Serialize};
use serde_with::TimestampSeconds;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct BlockInfoRequest {
    pub hash: BlockHash,

    // We only support json_block being true.
    #[clap(skip)]
    json_block: AlwaysTrue,
}

#[async_trait]
impl RPCRequest for &BlockInfoRequest {
    type Response = BlockInfoResponse;

    fn action(&self) -> &str {
        "block_info"
    }

    async fn call(&self, client: &RPCClient) -> Result<BlockInfoResponse> {
        client.rpc(self).await
    }
}

impl BlockInfoRequest {
    pub fn new(hash: BlockHash) -> Self {
        Self {
            hash,
            json_block: Default::default(),
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockInfoResponse {
    pub block_account: Address,
    pub amount: Raw,
    pub balance: Raw,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    pub height: u64,

    #[serde_as(as = "TimestampSeconds<String>")]
    pub local_timestamp: chrono::DateTime<Utc>,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    pub confirmed: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<Subtype>,

    pub contents: BlockHolder,
    // TODO: json_block
}

//TODO
/*#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use chrono::DateTime;
    use crate::Public;
    use crate::blocks::{Link, StateBlock};
    use crate::Address;

    #[test]
    fn decode() {
        let s = r#" {
            "block_account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
            "amount": "30000000000000000000000000000000000",
            "balance": "5606157000000000000000000000000000000",
            "height": "58",
            "local_timestamp": "0",
            "confirmed": "true",
            "contents": {
                "type": "state",
                "account": "nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est",
                "previous": "CE898C131AAEE25E05362F247760F8A3ACF34A9796A5AE0D9204E86B0637965E",
                "representative": "nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou",
                "balance": "5606157000000000000000000000000000000",
                "link": "5D1AA8A45F8736519D707FCB375976A7F9AF795091021D7E9C7548D6F45DD8D5",
                "link_as_account": "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z",
                "signature": "82D41BC16F313E4B2243D14DFFA2FB04679C540C2095FEE7EAE0F2F26880AD56DD48D87A7CC5DD760C5B2D76EE2C205506AA557BF00B60D8DEE312EC7343A501",
                "work": "8a142e07a10996d5"
            },
            "subtype": "send"
        }
        "#;

        let r = serde_json::from_str::<BlockInfoResponse>(s).unwrap();

        assert_eq!(
            r,
            BlockInfoResponse {
                block_account: Address::from_str("nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est").unwrap(),
                amount: Rai::from(30000000000000000000000000000000000),
                balance: Rai::from(5606157000000000000000000000000000000),
                height: 58,
                local_timestamp: DateTime::<Utc>::from_str("1970-01-01T00:00:00Z").unwrap(),
                confirmed: true,
                contents: BlockHolder::State(StateBlock::new(
                    Address::from_str("nano_1ipx847tk8o46pwxt5qjdbncjqcbwcc1rrmqnkztrfjy5k7z4imsrata9est").unwrap().to_public(),
                    BlockHash::from_str("CE898C131AAEE25E05362F247760F8A3ACF34A9796A5AE0D9204E86B0637965E").unwrap(),
                    Address::from_str("nano_1stofnrxuz3cai7ze75o174bpm7scwj9jn3nxsn8ntzg784jf1gzn1jjdkou").unwrap().to_public(),
                    Rai::from(5606157000000000000000000000000000000),
                    Link::DestinationAccount(Address::from_str("nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z").unwrap().to_public()),
                )),
                subtype: Some(BlockType::Send),
            }
        )
    }
}*/
