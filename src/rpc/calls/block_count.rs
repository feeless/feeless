use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct BlockCountRequest {}

#[async_trait]
impl RPCRequest for &BlockCountRequest {
    type Response = BlockCountResponse;

    fn action(&self) -> &str {
        "block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<BlockCountResponse> {
        client.rpc(self).await
    }
}

impl BlockCountRequest {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCountResponse {
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    count: u64,
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    unchecked: u64,
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    cemented: u64,
}
