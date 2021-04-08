use crate::{Address, Result, Rai};
use crate::rpc::client::{RPCClient, RPCRequest};
use serde::{Deserialize, Serialize};
use clap::Clap;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountsBalancesRequest {
    pub accounts: Vec<Address>, 
}

#[async_trait]
impl RPCRequest for &AccountsBalancesRequest {
    type Response = AccountsBalancesResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountsBalancesResponse> {
        client.rpc(self).await
    }
}

impl AccountsBalancesRequest {
    pub fn new(accounts: Vec<Address>) -> Self {
        Self {
            accounts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountsBalancesResponse {
    balances: HashMap<Address, HashMap<String, Rai>>,
}