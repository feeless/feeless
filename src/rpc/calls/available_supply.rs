use crate::rpc::client::{RPCClient, RPCRequest};
use crate::Raw;
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AvailableSupplyResponse {
    available: Raw,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let s = r#" {
            "available": "133248061996216572282917317807824970865"
        }
        "#;

        let r = serde_json::from_str::<AvailableSupplyResponse>(s).unwrap();

        assert_eq!(
            r,
            AvailableSupplyResponse {
                available: Raw::from(133248061996216572282917317807824970865),
            }
        )
    }
}
