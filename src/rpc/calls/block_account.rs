use crate::blocks::BlockHash;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Result};
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
        Self { hash }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockAccountResponse {
    account: Address,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#" {
            "account": "nano_1111111111111111111111111111111111111111111111111117353trpda"
        }
        "#;

        let r = serde_json::from_str::<BlockAccountResponse>(s).unwrap();

        assert_eq!(
            r,
            BlockAccountResponse {
                account: Address::from_str("nano_1111111111111111111111111111111111111111111111111117353trpda").unwrap(),
            }
        )
    }
}
