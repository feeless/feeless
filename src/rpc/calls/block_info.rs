use crate::blocks::{BlockHash, BlockHolder, Subtype};
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::{Address, Rai, Result};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfoResponse {
    pub block_account: Address,
    pub amount: Rai,
    pub balance: Rai,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    pub height: u64,

    #[serde_as(as = "TimestampSeconds<String>")]
    pub local_timestamp: chrono::DateTime<Utc>,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    pub confirmed: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<Subtype>,

    pub contents: BlockHolder,
}
