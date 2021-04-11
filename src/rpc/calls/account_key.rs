use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Public, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountKeyRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountKeyRequest {
    type Response = AccountKeyResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountKeyResponse> {
        client.rpc(self).await
    }
}

impl AccountKeyRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountKeyResponse {
    key: Public,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#" {
            "key": "3068BB1CA04525BB0E416C485FE6A67FD52540227D267CC8B6E8DA958A7FA039"
        }
        "#;

        let r = serde_json::from_str::<AccountKeyResponse>(s).unwrap();

        assert_eq!(
            r,
            AccountKeyResponse {
                key: Public::from_str(
                    "3068BB1CA04525BB0E416C485FE6A67FD52540227D267CC8B6E8DA958A7FA039"
                )
                .unwrap(),
            }
        )
    }
}
