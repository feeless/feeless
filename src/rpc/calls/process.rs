use crate::blocks::BlockHolder;
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::Result;
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct ProcessRequest {
    // We only support json_block being true.
    #[clap(skip)]
    json_block: AlwaysTrue,

    // pub subtype: Subtype,
    #[clap(subcommand)]
    pub block: BlockHolder,
}

#[async_trait]
impl RPCRequest for &ProcessRequest {
    type Response = ProcessResponse;

    fn action(&self) -> &str {
        "process"
    }

    async fn call(&self, client: &RPCClient) -> Result<ProcessResponse> {
        client.rpc(self).await
    }
}

impl ProcessRequest {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResponse {}
