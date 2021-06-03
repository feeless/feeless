use crate::blocks::BlockHash;
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Raw, Result};
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
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountInfoResponse {
    pub frontier: BlockHash,
    pub open_block: BlockHash,
    pub representative_block: BlockHash,
    pub balance: Raw,

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
    weight: Option<Raw>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pending: Option<Raw>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use std::str::FromStr;

    #[test]
    fn decode1() {
        let s = r#" {
            "frontier": "FF84533A571D953A596EA401FD41743AC85D04F406E76FDE4408EAED50B473C5",
            "open_block": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
            "representative_block": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
            "balance": "235580100176034320859259343606608761791",
            "modified_timestamp": "1501793775",
            "block_count": "33",
            "confirmation_height" : "28",
            "confirmation_height_frontier" : "34C70FCA0952E29ADC7BEE6F20381466AE42BD1CFBA4B7DFFE8BD69DF95449EB",
            "account_version": "1"
        }
        "#;

        let r = serde_json::from_str::<AccountInfoResponse>(s).unwrap();

        assert_eq!(
            r,
            AccountInfoResponse {
                frontier: BlockHash::from_str(
                    "FF84533A571D953A596EA401FD41743AC85D04F406E76FDE4408EAED50B473C5"
                )
                .unwrap(),
                open_block: BlockHash::from_str(
                    "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948"
                )
                .unwrap(),
                representative_block: BlockHash::from_str(
                    "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948"
                )
                .unwrap(),
                balance: Raw::from(235580100176034320859259343606608761791),
                modified_timestamp: DateTime::<Utc>::from_str("2017-08-03T20:56:15Z").unwrap(),
                block_count: 33,
                confirmation_height: 28,
                confirmation_height_frontier: BlockHash::from_str(
                    "34C70FCA0952E29ADC7BEE6F20381466AE42BD1CFBA4B7DFFE8BD69DF95449EB"
                )
                .unwrap(),
                account_version: 1,
                representative: None,
                weight: None,
                pending: None,
            }
        )
    }

    #[test]
    fn decode2() {
        // TODO: awaiting for clarification about the RPC command
        let s = r#" {
            "frontier": "FF84533A571D953A596EA401FD41743AC85D04F406E76FDE4408EAED50B473C5",
            "open_block": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
            "representative_block": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
            "balance": "235580100176034320859259343606608761791",
            "modified_timestamp": "1501793775",
            "block_count": "33",
            "confirmation_height" : "28",
            "confirmation_height_frontier" : "34C70FCA0952E29ADC7BEE6F20381466AE42BD1CFBA4B7DFFE8BD69DF95449EB",
            "account_version": "1",
            "representative": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
            "weight": "1105577030935649664609129644855132177",
            "pending": "2309370929000000000000000000000000"
        }
        "#;

        let r = serde_json::from_str::<AccountInfoResponse>(s).unwrap();

        assert_eq!(
            r,
            AccountInfoResponse {
                frontier: BlockHash::from_str(
                    "FF84533A571D953A596EA401FD41743AC85D04F406E76FDE4408EAED50B473C5"
                )
                .unwrap(),
                open_block: BlockHash::from_str(
                    "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948"
                )
                .unwrap(),
                representative_block: BlockHash::from_str(
                    "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948"
                )
                .unwrap(),
                balance: Raw::from(235580100176034320859259343606608761791),
                modified_timestamp: DateTime::<Utc>::from_str("2017-08-03T20:56:15Z").unwrap(),
                block_count: 33,
                confirmation_height: 28,
                confirmation_height_frontier: BlockHash::from_str(
                    "34C70FCA0952E29ADC7BEE6F20381466AE42BD1CFBA4B7DFFE8BD69DF95449EB"
                )
                .unwrap(),
                account_version: 1,
                representative: Some(
                    Address::from_str(
                        "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3"
                    )
                    .unwrap()
                ),
                weight: Some(Raw::from(1105577030935649664609129644855132177)),
                pending: Some(Raw::from(2309370929000000000000000000000000)),
            }
        )
    }
}
