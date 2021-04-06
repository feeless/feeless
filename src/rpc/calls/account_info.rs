use crate::blocks::BlockHash;
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use chrono::Utc;
use clap::Clap;
use serde::{Deserialize, Serialize};
use serde_with::TimestampSeconds;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountInfoRequest {
    pub account: Address,

    /// Do not request the account representative.
    #[clap(
        short,
        long = "no-representative",
        parse(from_flag = std::ops::Not::not)
    )]
    pub representative: bool,

    /// Do not request the account weight.
    #[clap(short, long = "no-weight", parse(from_flag = std::ops::Not::not))]
    pub weight: bool,

    /// Do not request the pending amount.
    #[clap(short, long = "no-pending", parse(from_flag = std::ops::Not::not))]
    pub pending: bool,
}

#[async_trait]
impl RPCRequest for &AccountInfoRequest {
    type Response = AccountInfoResponse;

    fn action(&self) -> &str {
        "account_info"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountInfoResponse> {
        client.rpc(self).await
    }
}

impl AccountInfoRequest {
    pub fn new(account: Address) -> Self {
        Self {
            account,
            weight: true,
            representative: true,
            pending: true,
        }
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

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    block_count: u64,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    confirmation_height: u64,

    confirmation_height_frontier: BlockHash,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    account_version: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    representative: Option<Address>,

    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<Rai>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pending: Option<Rai>,
}
