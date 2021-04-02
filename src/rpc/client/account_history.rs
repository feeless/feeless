use crate::rpc::client::{Client, RPCRequest};
use crate::{Address, Rai, Result};
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clap)]
pub struct AccountHistoryRequest {
    pub account: Address,

    #[clap(short, long, default_value = "-1")]
    pub count: i64,
}

impl RPCRequest for AccountHistoryRequest {
    fn action(&self) -> &str {
        "account_history"
    }
}

impl AccountHistoryRequest {
    pub fn new(account: Address, count: i64) -> Self {
        Self { account, count }
    }

    pub async fn call(&self, client: &Client) -> Result<AccountHistoryResponse> {
        client.rpc(self).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountHistoryResponse {
    pub balance: Rai,
    pub pending: Rai,
}
