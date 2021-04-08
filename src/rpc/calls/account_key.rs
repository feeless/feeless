use crate::{Public, Address, Result};
use crate::rpc::client::{RPCClient, RPCRequest};
use serde::{Deserialize, Serialize};
use clap::Clap;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountKeyRequest {
    pub account: Address, 
}

#[async_trait]
impl RPCRequest for &AccountKeyRequest {
    type Response = AccountKeyResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountKeyResponse> {
        client.rpc(self).await
    }
}

impl AccountKeyRequest {
    pub fn new(account: Address) -> Self {
        Self {
            account,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountKeyResponse {
    key: Public,
}