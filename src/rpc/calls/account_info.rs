use crate::blocks::BlockHash;
use crate::rpc::client::from_str;
use crate::rpc::client::{Client, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use chrono::Utc;
use clap::Clap;
use serde::{Deserialize, Serialize};
use serde_with::TimestampSeconds;

#[derive(Debug, Serialize, Clap)]
pub struct AccountInfoRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountInfoRequest {
    type Response = AccountInfoResponse;

    fn action(&self) -> &str {
        "account_info"
    }

    async fn call(&self, client: &Client) -> Result<AccountInfoResponse> {
        client.rpc(self).await
    }
}

impl AccountInfoRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfoResponse {
    pub frontier: BlockHash,
    pub open_block: BlockHash,
    pub representative_block: BlockHash,
    pub balance: Rai,

    #[serde_as(as = "TimestampSeconds<String>")]
    pub modified_timestamp: chrono::DateTime<Utc>,

    #[serde(deserialize_with = "from_str")]
    block_count: u64,

    #[serde(deserialize_with = "from_str")]
    confirmation_height: u64,

    confirmation_height_frontier: BlockHash,

    #[serde(deserialize_with = "from_str")]
    account_version: u64,
}
