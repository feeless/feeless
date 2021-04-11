use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountRepresentativeRequest {
    pub account: Address,
}

#[async_trait]
impl RPCRequest for &AccountRepresentativeRequest {
    type Response = AccountRepresentativeResponse;

    fn action(&self) -> &str {
        "account_representative"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountRepresentativeResponse> {
        client.rpc(self).await
    }
}

impl AccountRepresentativeRequest {
    pub fn new(account: Address) -> Self {
        Self { account }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountRepresentativeResponse {
    representative: Address,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#" {
            "representative" : "nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5"
        }
        "#;

        let r = serde_json::from_str::<AccountRepresentativeResponse>(s).unwrap();

        assert_eq!(
            r,
            AccountRepresentativeResponse {
                representative: Address::from_str("nano_16u1uufyoig8777y6r8iqjtrw8sg8maqrm36zzcm95jmbd9i9aj5i8abr8u5").unwrap(),
            }
        )
    }
}
