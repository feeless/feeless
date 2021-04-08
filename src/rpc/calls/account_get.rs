use crate::{Public, Address, Result};
use crate::rpc::client::{RPCClient, RPCRequest};
use serde::{Deserialize, Serialize};
use clap::Clap;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountGetRequest {
    pub key: Public, 
}

#[async_trait]
impl RPCRequest for &AccountGetRequest {
    type Response = AccountGetResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountGetResponse> {
        client.rpc(self).await
    }
}

impl AccountGetRequest {
    pub fn new(key: Public) -> Self {
        Self {
            key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountGetResponse {
    account: Address,
}