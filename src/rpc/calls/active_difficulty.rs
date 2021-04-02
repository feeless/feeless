use crate::blocks::{BlockHash, BlockType, StateBlock};
use crate::pow::Difficulty;
use crate::rpc::calls::from_str;
use crate::rpc::client::{Client, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::{Address, Rai, Result, Signature, Work};
use async_trait::async_trait;
use chrono::Utc;
use clap::Clap;
use serde::{Deserialize, Serialize};
use serde_with::TimestampSeconds;

#[derive(Debug, Serialize, Clap)]
pub struct ActiveDifficultyRequest {}

#[async_trait]
impl RPCRequest for &ActiveDifficultyRequest {
    type Response = ActiveDifficultyResponse;

    fn action(&self) -> &str {
        "active_difficulty"
    }

    async fn call(&self, client: &Client) -> Result<ActiveDifficultyResponse> {
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
