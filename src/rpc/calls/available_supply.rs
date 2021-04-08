use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Rai;
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
// should it be pub struct AvailableSupplyRequest;?
pub struct AvailableSupplyRequest {}

#[async_trait]
impl RPCRequest for &AvailableSupplyRequest {
    type Response = AvailableSupplyResponse;

    fn action(&self) -> &str {
        "available_supply"
    }

    async fn call(&self, client: &RPCClient) -> Result<AvailableSupplyResponse> {
        client.rpc(self).await
    }
}

impl AvailableSupplyRequest {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableSupplyResponse {
    available: Rai,
}
