use crate::blocks::BlockHash;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountsFrontiersRequest {
    pub accounts: Vec<Address>,
}

#[async_trait]
impl RPCRequest for &AccountsFrontiersRequest {
    type Response = AccountsFrontiersResponse;

    fn action(&self) -> &str {
        "accounts_frontiers"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountsFrontiersResponse> {
        client.rpc(self).await
    }
}

impl AccountsFrontiersRequest {
    pub fn new(accounts: Vec<Address>) -> Self {
        Self { accounts }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountsFrontiersResponse {
    frontiers: HashMap<Address, BlockHash>,
}
