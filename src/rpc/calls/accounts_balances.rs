use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Rai, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AccountsBalancesRequest {
    pub accounts: Vec<Address>,
}

#[async_trait]
impl RPCRequest for &AccountsBalancesRequest {
    type Response = AccountsBalancesResponse;

    fn action(&self) -> &str {
        "accounts_balances"
    }

    async fn call(&self, client: &RPCClient) -> Result<AccountsBalancesResponse> {
        client.rpc(self).await
    }
}

impl AccountsBalancesRequest {
    pub fn new(accounts: Vec<Address>) -> Self {
        Self { accounts }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountsBalancesResponse {
    balances: HashMap<Address, AccountsBalancesEntry>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountsBalancesEntry {
    balance: Rai,
    pending: Rai,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#" {
            "balances" : {
                "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7":
                {
                    "balance": "10000",
                    "pending": "10000"
                },
                "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7":
                {
                    "balance": "10000000",
                    "pending": "0"
                }
            }
        }
        "#;

        let r = serde_json::from_str::<AccountsBalancesResponse>(s).unwrap();

        let mut balances: HashMap<Address, AccountsBalancesEntry> = HashMap::new();

        balances.insert(Address::from_str("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7").unwrap(),
        AccountsBalancesEntry {
            balance: Rai::from(10000),
            pending: Rai::from(10000),
        });

        balances.insert(Address::from_str("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7").unwrap(),
        AccountsBalancesEntry {
            balance: Rai::from(10000000),
            pending: Rai::from(0),
        });

        assert_eq!(
            r,
            AccountsBalancesResponse {
                balances,
            }
        )
    }
}
