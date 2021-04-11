use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountWeightRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountWeightRequest {
    type Response = AccountWeightResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountWeightResponse> {
        client.rpc(self).await
    }
}

impl AccountWeightRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountWeightResponse {
    weight: Rai,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let s = r#" {
            "weight": "10000"
        }
        "#;

        let r = serde_json::from_str::<AccountWeightResponse>(s).unwrap();

        assert_eq!(
            r,
            AccountWeightResponse {
                weight: Rai::from(10000),
            }
        )
    }
}
