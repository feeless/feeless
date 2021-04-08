use crate::blocks::BlockHash;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate:: {Result, Address};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct BlockAccountRequest {
    hash: BlockHash,
}

#[async_trait]
impl RPCRequest for &BlockAccountRequest {
    type Response = BlockAccountResponse;

    fn action(&self) -> &str {
        "block_account"
    }

    async fn call(&self, client: &RPCClient) -> Result<BlockAccountResponse> {
        client.rpc(self).await
    }
}

impl BlockAccountRequest {
    pub fn new(hash: BlockHash) -> Self {
        Self {
            hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockAccountResponse {
    account: Address,
}