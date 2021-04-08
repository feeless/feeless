use crate::blocks::BlockHash;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct BlockConfirmRequest {
    hash: BlockHash,
}

#[async_trait]
impl RPCRequest for &BlockConfirmRequest {
    type Response = BlockConfirmResponse;

    fn action(&self) -> &str {
        "block_confirm"
    }

    async fn call(&self, client: &RPCClient) -> Result<BlockConfirmResponse> {
        client.rpc(self).await
    }
}

impl BlockConfirmRequest {
    pub fn new(hash: BlockHash) -> Self {
        Self { hash }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockConfirmResponse {
    started: u8,
}
