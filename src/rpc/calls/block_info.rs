use crate::blocks::{BlockHash, BlockType, StateBlock};
use crate::rpc::calls::from_str;
use crate::rpc::client::{Client, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::{Address, Rai, Result, Signature, Work};
use async_trait::async_trait;
use chrono::Utc;
use clap::Clap;
use serde::{Deserialize, Serialize};
use serde_with::TimestampSeconds;

#[derive(Debug, Serialize, Clap)]
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

    async fn call(&self, client: &Client) -> Result<BlockInfoResponse> {
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

    #[serde(deserialize_with = "from_str")]
    pub height: u64,

    #[serde_as(as = "TimestampSeconds<String>")]
    pub local_timestamp: chrono::DateTime<Utc>,

    #[serde(deserialize_with = "from_str")]
    pub confirmed: bool,

    pub subtype: BlockType,

    pub contents: BlockContents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockContents {
    #[serde(rename = "type")]
    pub block_type: BlockType,

    pub account: Address,
    pub previous: BlockHash,
    pub representative: Address,
    pub balance: Rai,
    pub link: BlockHash, // TODO: Link
    pub link_as_account: Address,
    pub signature: Signature,
    pub work: Work,
}
