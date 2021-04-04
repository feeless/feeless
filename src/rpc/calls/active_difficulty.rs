use crate::pow::Difficulty;
use crate::rpc::calls::{as_str, from_str};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct ActiveDifficultyRequest {}

#[async_trait]
impl RPCRequest for &ActiveDifficultyRequest {
    type Response = ActiveDifficultyResponse;

    fn action(&self) -> &str {
        "active_difficulty"
    }

    async fn call(&self, client: &RPCClient) -> Result<ActiveDifficultyResponse> {
        client.rpc(self).await
    }
}

impl ActiveDifficultyRequest {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ActiveDifficultyResponse {
    #[serde(deserialize_with = "from_str", serialize_with = "as_str")]
    pub multiplier: f64,

    pub network_current: Difficulty,
    pub network_minimum: Difficulty,
    pub network_receive_current: Difficulty,
    pub network_receive_minimum: Difficulty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response() {
        let r1 = ActiveDifficultyResponse {
            multiplier: 0.1,
            network_current: Difficulty::new(1),
            network_minimum: Difficulty::new(2),
            network_receive_current: Difficulty::new(3),
            network_receive_minimum: Difficulty::new(4),
        };
        let json = serde_json::to_string(&r1).unwrap();
        let r2 = serde_json::from_str(&json).unwrap();
        assert_eq!(r1, r2);
    }
}
