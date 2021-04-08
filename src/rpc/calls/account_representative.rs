use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountRepresentativeRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountRepresentativeRequest {
    type Response = AccountRepresentativeResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountRepresentativeResponse> {
        client.rpc(self).await
    }
}

impl AccountRepresentativeRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRepresentativeResponse {
    representative: Address,
}
