use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountBlockCountRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountBlockCountRequest {
    type Response = AccountBlockCountResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountBlockCountResponse> {
        client.rpc(self).await
    }
}

impl AccountBlockCountRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountBlockCountResponse {
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    block_count: u64,
}
