use crate::pow::Difficulty;
use crate::rpc::calls::from_str;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clap)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveDifficultyResponse {
    #[serde(deserialize_with = "from_str")]
    pub multiplier: f64,

    pub network_current: Difficulty,
    pub network_minimum: Difficulty,
    pub network_receive_current: Difficulty,
    pub network_receive_minimum: Difficulty,
}
