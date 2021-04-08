use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountWeightRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountWeightRequest {
    type Response = AccountWeightResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountWeightResponse> {
        client.rpc(self).await
    }
}

impl AccountWeightRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWeightResponse {
    weight: Rai,
}
