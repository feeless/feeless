use crate::blocks::BlockHash;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountsFrontiersRequest {
    pub accounts: Vec<Address>,
}

#[async_trait]
impl RPCRequest for &AccountsFrontiersRequest {
    type Response = AccountsFrontiersResponse;

    fn action(&self) -> &str {
        "accounts_frontiers"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountsFrontiersResponse> {
        client.rpc(self).await
    }
}

impl AccountsFrontiersRequest {
    pub fn new(accounts: Vec<Address>) -> Self {
        Self { accounts }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountsFrontiersResponse {
    frontiers: HashMap<Address, BlockHash>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#" {
            "frontiers" : {
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": "791AF413173EEE674A6FCF633B5DFC0F3C33F397F0DA08E987D9E0741D40D81A",
                "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7": "6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F"
            }
        }
        "#;

        let r = serde_json::from_str::<AccountsFrontiersResponse>(s).unwrap();

        let mut frontiers: HashMap<Address, BlockHash> = HashMap::new();

        frontiers.insert(Address::from_str("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3").unwrap(),
        BlockHash::from_str("791AF413173EEE674A6FCF633B5DFC0F3C33F397F0DA08E987D9E0741D40D81A").unwrap());
        
        frontiers.insert(Address::from_str("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7").unwrap(),
        BlockHash::from_str("6A32397F4E95AF025DE29D9BF1ACE864D5404362258E06489FABDBA9DCCC046F").unwrap());

        assert_eq!(
            r,
            AccountsFrontiersResponse {
                frontiers,
            }
        )
    }
}
