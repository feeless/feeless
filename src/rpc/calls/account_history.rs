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
pub struct AccountHistoryRequest {
    pub account: Address,

    // We only support raw.
    #[clap(skip)]
    raw: bool,

    /// Limit the number of results to `count`.
    #[clap(short, long, default_value = "-1")]
    pub count: i64,

    /// Start displaying blocks from this hash. Useful for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    head: Option<BlockHash>,

    /// Skips a number of blocks starting from head.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    offset: Option<u64>,

    /// Request to reverse the results.
    #[clap(short, long)]
    reverse: bool,

    /// Results will be filtered to only show sends/receives connected to the provided account(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    account_filter: Option<Vec<Address>>,
}

#[async_trait]
impl RPCRequest for &AccountHistoryRequest {
    type Response = AccountHistoryResponse;

    fn action(&self) -> &str {
        "account_history"
    }

    async fn call(&self, client: &Client) -> Result<Self::Response> {
        client.rpc(self).await
    }
}

impl AccountHistoryRequest {
    pub fn new(account: Address, count: i64) -> Self {
        Self {
            account,
            count,
            raw: false,
            head: None,
            offset: None,
            reverse: false,
            account_filter: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountHistoryResponse {
    pub account: Address,
    pub history: Vec<AccountHistoryEntry>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<BlockHash>,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountHistoryEntry {
    #[serde(rename = "type")]
    pub block_type: String,

    pub account: Address,
    pub amount: Rai,

    #[serde_as(as = "TimestampSeconds<String>")]
    pub local_timestamp: chrono::DateTime<Utc>,

    #[serde(deserialize_with = "from_str")]
    pub height: u64,

    pub hash: BlockHash,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#"
        {
            "account":"nano_3x4ui45q1cw8hydmfdn4ec5ijsdqi4ryp14g4ayh71jcdkwmddrq7ca9xzn9",
            "history":[{
                "type":"send",
                "account":"nano_3jwrszth46rk1mu7rmb4rhm54us8yg1gw3ipodftqtikf5yqdyr7471nsg1k",
                "amount":"1500000000000000000000000000000000001",
                "local_timestamp":"1614327355",
                "height":"39",
                "hash":"721BF781D07CEB0072C6BA8C9B5ADA6593F8F6E6DAA4B60889A1DDC2DFA553E2"
            }],
            "previous":"180938FFFD9E89DDA7B02F641D690DA8BF4ED8BB9ABCBCB294E6219A4FBE76E7"
        }
        "#;

        let r = serde_json::from_str::<AccountHistoryResponse>(s).unwrap();
        // let a = DateTime::<Utc>::from_str().unwrap();
        assert_eq!(
            r,
            AccountHistoryResponse {
                account: Address::from_str(
                    "nano_3x4ui45q1cw8hydmfdn4ec5ijsdqi4ryp14g4ayh71jcdkwmddrq7ca9xzn9"
                )
                .unwrap(),
                history: vec![AccountHistoryEntry {
                    block_type: "send".into(),
                    account: Address::from_str(
                        "nano_3jwrszth46rk1mu7rmb4rhm54us8yg1gw3ipodftqtikf5yqdyr7471nsg1k"
                    )
                    .unwrap(),
                    amount: Rai::new(1500000000000000000000000000000000001u128),
                    local_timestamp: DateTime::<Utc>::from_str("2021-02-26T08:15:55Z").unwrap(),
                    height: 39,
                    hash: BlockHash::from_str(
                        "721BF781D07CEB0072C6BA8C9B5ADA6593F8F6E6DAA4B60889A1DDC2DFA553E2"
                    )
                    .unwrap(),
                },],
                previous: Some(
                    BlockHash::from_str(
                        "180938FFFD9E89DDA7B02F641D690DA8BF4ED8BB9ABCBCB294E6219A4FBE76E7"
                    )
                    .unwrap()
                ),
            }
        );
    }
}
