use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountsBalancesRequest {
    pub accounts: Vec<Address>,
}

#[async_trait]
impl RPCRequest for &AccountsBalancesRequest {
    type Response = AccountsBalancesResponse;

    fn action(&self) -> &str {
        "accounts_balances"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountsBalancesResponse> {
        client.rpc(self).await
    }
}

impl AccountsBalancesRequest {
    pub fn new(accounts: Vec<Address>) -> Self {
        Self { accounts }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountsBalancesResponse {
    balances: HashMap<Address, HashMap<String, Rai>>,
}
