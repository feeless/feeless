use crate::blocks::BlockHash;
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::{Difficulty, Result, Work};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct WorkValidateRequest {
    pub hash: BlockHash,
    pub work: Work,
}

#[async_trait]
impl RPCRequest for &WorkValidateRequest {
    type Response = WorkValidateResponse;

    fn action(&self) -> &str {
        "work_validate"
    }

    async fn call(&self, client: &RPCClient) -> Result<WorkValidateResponse> {
        client.rpc(self).await
    }
}

impl WorkValidateRequest {
    pub fn new(work: Work, hash: BlockHash) -> Self {
        Self { work, hash }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkValidateResponse {
    // TODO: This is meant to be a bool as a number in a string?
    valid_all: String,
    valid_receive: String,
    difficulty: Difficulty,

    // TODO: Make multiplier a type? It's used in multiple areas.
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    multiplier: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn decode() {
        let s = r#"
        {
            "valid_all": "1",
            "valid_receive": "1",
            "difficulty": "fffffff93c41ec94",
            "multiplier": "1.182623871097636"
        }
        "#;

        let r = serde_json::from_str::<WorkValidateResponse>(s).unwrap();
        assert_eq!(
            r,
            WorkValidateResponse {
                valid_all: String::from("1"),
                valid_receive: String::from("1"),
                difficulty: Difficulty::from_str("fffffff93c41ec94").unwrap(),
                multiplier: 1.182623871097636,
            }
        );
    }
}
