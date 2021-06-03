use crate::blocks::BlockHash;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Address, Raw, Result};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clap, Clone)]
pub struct AccountsPendingRequest {
    accounts: Vec<Address>,

    /// Limit the number of results to `count`.
    #[clap(short, long, default_value = "1")]
    count: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    threshold: Option<Raw>,

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
    },
    Threshold {
        blocks: HashMap<Address, HashMap<BlockHash, Raw>>,
    },
    Source {
        blocks: HashMap<Address, HashMap<BlockHash, BlockEntry>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BlockEntry {
    amount: Raw,
    source: Address,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode1() {
        let s = r#" {
            "blocks" : {
                "nano_1111111111111111111111111111111111111111111111111117353trpda": ["142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D"],
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": ["4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74"]
            }
        }  
        "#;

        let r = serde_json::from_str::<AccountsPendingResponse>(s).unwrap();

        let mut blocks: HashMap<Address, Vec<BlockHash>> = HashMap::new();
        blocks.insert(
            Address::from_str("nano_1111111111111111111111111111111111111111111111111117353trpda")
                .unwrap(),
            vec![BlockHash::from_str(
                "142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D",
            )
            .unwrap()],
        );
        blocks.insert(
            Address::from_str("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3")
                .unwrap(),
            vec![BlockHash::from_str(
                "4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74",
            )
            .unwrap()],
        );

        assert_eq!(r, AccountsPendingResponse::OnlyBlockHash { blocks });
    }

    #[test]
    fn decode2() {
        let s = r#" {
            "blocks" : {
                "nano_1111111111111111111111111111111111111111111111111117353trpda": {
                    "142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D": "6000000000000000000000000000000"
                },
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": {
                    "4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74": "106370018000000000000000000000000"
                }
            }
        }
        "#;

        let r = serde_json::from_str::<AccountsPendingResponse>(s).unwrap();

        let mut blocks: HashMap<Address, HashMap<BlockHash, Raw>> = HashMap::new();
        let mut threshold1: HashMap<BlockHash, Raw> = HashMap::new();
        let mut threshold2: HashMap<BlockHash, Raw> = HashMap::new();
        threshold1.insert(
            BlockHash::from_str("142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D")
                .unwrap(),
            Raw::from(6000000000000000000000000000000),
        );
        threshold2.insert(
            BlockHash::from_str("4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74")
                .unwrap(),
            Raw::from(106370018000000000000000000000000),
        );
        blocks.insert(
            Address::from_str("nano_1111111111111111111111111111111111111111111111111117353trpda")
                .unwrap(),
            threshold1,
        );
        blocks.insert(
            Address::from_str("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3")
                .unwrap(),
            threshold2,
        );

        assert_eq!(r, AccountsPendingResponse::Threshold { blocks });
    }

    #[test]
    fn decode3() {
        let s = r#" {
            "blocks" : {
                "nano_1111111111111111111111111111111111111111111111111117353trpda": {
                    "142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D": {
                    "amount": "6000000000000000000000000000000",
                    "source": "nano_3dcfozsmekr1tr9skf1oa5wbgmxt81qepfdnt7zicq5x3hk65fg4fqj58mbr"
                    }
                },
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3": {
                    "4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74": {
                    "amount": "106370018000000000000000000000000",
                    "source": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo"
                    }
                }
            }
        }
        "#;

        let r = serde_json::from_str::<AccountsPendingResponse>(s).unwrap();

        let mut blocks: HashMap<Address, HashMap<BlockHash, BlockEntry>> = HashMap::new();
        let mut threshold1: HashMap<BlockHash, BlockEntry> = HashMap::new();
        let mut threshold2: HashMap<BlockHash, BlockEntry> = HashMap::new();
        threshold1.insert(
            BlockHash::from_str("142A538F36833D1CC78B94E11C766F75818F8B940771335C6C1B8AB880C5BB1D")
                .unwrap(),
            BlockEntry {
                amount: Raw::from(6000000000000000000000000000000),
                source: Address::from_str(
                    "nano_3dcfozsmekr1tr9skf1oa5wbgmxt81qepfdnt7zicq5x3hk65fg4fqj58mbr",
                )
                .unwrap(),
            },
        );
        threshold2.insert(
            BlockHash::from_str("4C1FEEF0BEA7F50BE35489A1233FE002B212DEA554B55B1B470D78BD8F210C74")
                .unwrap(),
            BlockEntry {
                amount: Raw::from(106370018000000000000000000000000),
                source: Address::from_str(
                    "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                )
                .unwrap(),
            },
        );
        blocks.insert(
            Address::from_str("nano_1111111111111111111111111111111111111111111111111117353trpda")
                .unwrap(),
            threshold1,
        );
        blocks.insert(
            Address::from_str("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3")
                .unwrap(),
            threshold2,
        );

        assert_eq!(r, AccountsPendingResponse::Source { blocks });
    }
}
