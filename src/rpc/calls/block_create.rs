use crate::blocks::{BlockHash, BlockType, Link, StateBlock};
use crate::rpc::client::{RPCClient, RPCRequest};
use crate::rpc::AlwaysTrue;
use crate::wallet::WalletId;
use crate::{Address, Difficulty, Private, Raw, Result, Work};
use async_trait::async_trait;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct BlockCreateRequest {
    // We only support json_block being true.
    #[clap(skip)]
    json_block: AlwaysTrue,

    /// Specify the block type. It currently only makes sense to use `state` for new blocks.
    #[clap(short = 't', long, default_value = "state")]
    #[serde(rename = "type")]
    block_type: BlockType,

    /// Final balance for account after block creation.
    #[clap(short, long)]
    pub balance: Raw,

    /// The wallet ID that the account the block is being created for is in.
    #[clap(short = 'i', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<WalletId>,

    /// The account the block is being created for.
    #[clap(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<Address>,

    /// Instead of using "wallet" & "account" parameters, you can directly pass in a private key.
    #[clap(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<Private>,

    /// The block hash of the source of funds for this receive block
    #[clap(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Private>,

    /// The account that the sent funds should be accessible to.
    #[clap(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<Private>,

    /// Instead of using "source" & "destination" parameters, you can directly pass "link".
    /// Source block hash to receive or destination public key to send.
    #[clap(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Link>,

    /// The account that block account will use as its representative.
    #[clap(short, long)]
    pub representative: Address,

    /// The block hash of the previous block on this account's block chain.
    /// Do not specify if it's for the first block in the account.
    #[clap(
        short,
        long,
        default_value = "0000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub previous: BlockHash,

    /// Specify own work.
    #[clap(short, long, group = "work_or_difficulty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work: Option<Work>,

    /// Uses difficulty value to generate work. Only used if optional work is not given.
    #[clap(short = 'f', long, group = "work_or_difficulty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,
}

#[async_trait]
impl RPCRequest for &BlockCreateRequest {
    type Response = BlockCreateResponse;

    fn action(&self) -> &str {
        "block_create"
    }

    async fn call(&self, client: &RPCClient) -> Result<BlockCreateResponse> {
        client.rpc(self).await
    }
}

impl BlockCreateRequest {
    pub fn new(
        block_type: BlockType,
        balance: Raw,
        representative: Address,
        previous: BlockHash,
    ) -> Self {
        Self {
            json_block: Default::default(),
            block_type,
            balance,
            wallet: None,
            account: None,
            key: None,
            source: None,
            destination: None,
            link: None,
            representative,
            previous,
            work: None,
            difficulty: None,
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCreateResponse {
    hash: BlockHash,
    difficulty: Difficulty,
    block: StateBlock,
}
