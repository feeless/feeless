use crate::rpc::client::{Client, RPCRequest};
use crate::{Address, Rai, Result};
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clap)]
pub struct AccountBalanceRequest {
    pub account: Address,
}

impl RPCRequest for AccountBalanceRequest {
    fn action(&self) -> &str {
        "account_balance"
    }
}

impl AccountBalanceRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }

    pub async fn call(&self, client: &Client) -> Result<AccountBalanceResponse> {
        client.rpc(self).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountBalanceResponse {
    pub balance: Rai,
    pub pending: Rai,
}
