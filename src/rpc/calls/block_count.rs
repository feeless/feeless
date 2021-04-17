use crate::rpc::calls::{as_str, as_str_option, from_str, from_str_option};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct BlockCountRequest {
    #[clap(long)]
    include_cemented: bool,
}

#[async_trait]
impl RPCRequest for &BlockCountRequest {
    type Response = BlockCountResponse;

    fn action(&self) -> &str {
        "block_count"
    }

    async fn call(&self, client: &RPCClient) -> Result<BlockCountResponse> {
        client.rpc(self).await
    }
}

impl BlockCountRequest {
    pub fn new() -> Self {
        Self {
            include_cemented: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockCountResponse {
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    count: u64,

    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    unchecked: u64,

    #[serde(default)]
    #[serde(serialize_with = "as_str_option", deserialize_with = "from_str_option")]
    cemented: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode1() {
        let s = r#" {
            "count": "1000",
            "unchecked": "10"
        }
        "#;

        let r = serde_json::from_str::<BlockCountResponse>(s).unwrap();

        assert_eq!(
            r,
            BlockCountResponse {
                count: 1000,
                unchecked: 10,
                cemented: None,
            }
        );
    }

    #[test]
    fn decode2() {
        let s = r#" {
            "count": "1000",
            "unchecked": "10",
            "cemented": "25"
        }
        "#;

        let r = serde_json::from_str::<BlockCountResponse>(s).unwrap();

        assert_eq!(
            r,
            BlockCountResponse {
                count: 1000,
                unchecked: 10,
                cemented: Some(25),
            }
        );
    }
}
