use std::net::SocketAddr;

use crate::node::cookie::Cookie;
use crate::node::network::Network;
use crate::node::state::State;

use crate::{Block, BlockHash, Public, Raw};
use anyhow::Context;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MemoryState {
    network: Network,
    blocks: HashMap<BlockHash, Block>,
    block_hash_to_account: HashMap<BlockHash, Public>,
    latest_block_hash: HashMap<Public, BlockHash>,
}

impl MemoryState {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            blocks: HashMap::new(),
            block_hash_to_account: HashMap::new(),
            latest_block_hash: HashMap::new(),
        }
    }
}

#[async_trait]
impl State for MemoryState {
    fn network(&self) -> Network {
        self.network
    }

    async fn add_block(&mut self, account: &Public, full_block: &Block) -> anyhow::Result<()> {
        self.blocks.insert(
            full_block.hash().context("Add block")?.to_owned(),
            full_block.to_owned(),
        );
        self.block_hash_to_account
            .insert(full_block.hash()?.to_owned(), account.to_owned());
        self.latest_block_hash
            .insert(account.to_owned(), full_block.hash()?.to_owned());
        Ok(())
    }

    async fn get_block_by_hash(&self, hash: &BlockHash) -> anyhow::Result<Option<Block>> {
        Ok(self.blocks.get(hash).map(|b| b.to_owned()))
    }

    async fn get_latest_block_hash_for_account(
        &self,
        account: &Public,
    ) -> anyhow::Result<Option<BlockHash>> {
        Ok(self.latest_block_hash.get(account).map(|b| b.to_owned()))
    }

    async fn account_for_block_hash(
        &mut self,
        block_hash: &BlockHash,
    ) -> Result<Option<Public>, anyhow::Error> {
        Ok(self
            .block_hash_to_account
            .get(block_hash)
            .map(|a| a.to_owned()))
    }

    async fn set_cookie(
        &mut self,
        _socket_addr: SocketAddr,
        _cookie: Cookie,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    async fn cookie_for_socket_addr(
        &self,
        _socket_addr: &SocketAddr,
    ) -> Result<Option<Cookie>, anyhow::Error> {
        unimplemented!()
    }
}
