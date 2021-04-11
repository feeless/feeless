use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Public, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountGetRequest {
    pub key: Public,
}

#[async_trait]
impl RPCRequest for &AccountGetRequest {
    type Response = AccountGetResponse;

    fn action(&self) -> &str {
        "account_block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountGetResponse> {
        client.rpc(self).await
    }
}

impl AccountGetRequest {
    pub fn new(key: Public) -> Self {
        Self { key }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountGetResponse {
    account: Address,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#" {
            "account" : "nano_1e5aqegc1jb7qe964u4adzmcezyo6o146zb8hm6dft8tkp79za3sxwjym5rx"
        }
        "#;

        let r = serde_json::from_str::<AccountGetResponse>(s).unwrap();

        assert_eq!(
            r,
            AccountGetResponse {
                account: Address::from_str("nano_1e5aqegc1jb7qe964u4adzmcezyo6o146zb8hm6dft8tkp79za3sxwjym5rx").unwrap(),
            }
        )
    }
}
