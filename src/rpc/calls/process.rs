use crate::blocks::{deserialize_to_unsure_link, BlockType};
use crate::blocks::{BlockHash, Link, Subtype};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::{Address, Rai, Result, Signature, Work};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Clap)]
pub struct StateBlockRequest {
    #[clap(short = 't', long, default_value = "state")]
    #[serde(rename = "type")]
    pub block_type: BlockType,

    #[clap(short, long)]
    pub account: Address,

    #[clap(short, long)]
    pub previous: BlockHash,

    #[clap(short, long)]
    pub representative: Address,

    #[clap(short, long)]
    pub balance: Rai,

    #[serde(deserialize_with = "deserialize_to_unsure_link")]
    #[clap(short, long)]
    pub link: Link,

    #[clap(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work: Option<Work>,

    #[clap(short = 'g', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct ProcessRequest {
    // We only support json_block being true.
    #[clap(skip)]
    json_block: AlwaysTrue,

    pub subtype: Subtype,

    #[clap(flatten)]
    pub block: StateBlockRequest,
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
