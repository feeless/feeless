use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountBalanceRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountBalanceRequest {
    type Response = AccountBalanceResponse;

    fn action(&self) -> &str {
        "account_balance"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountBalanceResponse> {
        client.rpc(self).await
    }
}

impl AccountBalanceRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountBalanceResponse {
    pub balance: Rai,
    pub pending: Rai,
}
