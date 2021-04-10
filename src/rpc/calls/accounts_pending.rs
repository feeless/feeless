use crate::{Address, Rai, Result};
use crate::rpc::client::{RPCRequest, RPCClient};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::blocks::BlockHash;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap, Clone)]
pub struct AccountsPendingRequest {
    accounts: Vec<Address>,

    /// Limit the number of results to `count`.
    #[clap(short, long, default_value = "1")]
    count: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    threshold: Option<Rai>,

    #[clap(long)]
    source: bool,

    #[clap(long)]
    include_active: bool,

    #[clap(long)]
    sorting: bool,

    #[clap(long)]
    include_only_confirmed: bool,
}

#[async_trait]
impl RPCRequest for &AccountsPendingRequest {
    type Response = AccountsPendingResponse;

    fn action(&self) -> &str {
        "accounts_pending"
    }

    async fn call(&self, client: &RPCClient) -> Result<Self::Response> {
        client.rpc(self).await
    }
}

impl AccountsPendingRequest {
    pub fn new(accounts: Vec<Address>, count: u64) -> Self {
        Self {
            accounts,
            count,
            threshold: None,
            source: false,
            include_active: false,
            sorting: false,
            include_only_confirmed: false,
        }
    } 
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum AccountsPendingResponse {
    OnlyBlockHash {
        blocks: HashMap<Address, Vec<BlockHash>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode1() {
        let s = r#" {
            "blocks" : {
              "nano_1111111111111111111111111111111111111111111111111117353trpda": ["142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D"]
            }
        }  
        "#;

        let r = serde_json::from_str::<AccountsPendingResponse>(s).unwrap();

        let mut blocks: HashMap<Address, Vec<BlockHash>> = HashMap::new();
        blocks.insert(Address::from_str("nano_1111111111111111111111111111111111111111111111111117353trpda").unwrap(),
        vec![BlockHash::from_str("142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D").unwrap()]);

        assert_eq!(
            r,
            AccountsPendingResponse::OnlyBlockHash {
                blocks,
            }
        );
    }
}