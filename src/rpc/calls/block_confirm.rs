use crate::blocks::BlockHash;
use crate::rpc::calls::{as_str, from_str};
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockConfirmResponse {
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    started: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let s = r#" {
            "started": "1"
        }
        "#;

        let r = serde_json::from_str::<BlockConfirmResponse>(s).unwrap();

        assert_eq!(r, BlockConfirmResponse { started: 1 })
    }
}
