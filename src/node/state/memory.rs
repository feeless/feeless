use std::net::SocketAddr;

use crate::blocks::Block;
use crate::node::cookie::Cookie;
use crate::node::network::Network;
use crate::node::state::State;
use crate::pow::work::Subject::Hash;
use crate::{BlockHash, FullBlock, Public, Raw};
use anyhow::Context;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MemoryState {
    network: Network,
    blocks: HashMap<BlockHash, FullBlock>,
    account_balances: HashMap<Public, Raw>,
    block_hash_to_account: HashMap<BlockHash, Public>,
}

impl MemoryState {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            blocks: HashMap::new(),
            account_balances: HashMap::new(),
            block_hash_to_account: HashMap::new(),
        }
    }
}

#[async_trait]
impl State for MemoryState {
    fn network(&self) -> Network {
        self.network
    }

    async fn add_block(&mut self, account: &Public, full_block: &FullBlock) -> anyhow::Result<()> {
        self.blocks.insert(
            full_block.hash().context("Add block")?,
            full_block.to_owned(),
        );
        self.block_hash_to_account
            .insert(full_block.hash()?, account.to_owned());
        Ok(())
    }

    async fn get_block_by_hash(&mut self, hash: &BlockHash) -> anyhow::Result<Option<FullBlock>> {
        Ok(self.blocks.get(hash).map(|b| b.to_owned()))
    }

    async fn account_balance(&mut self, account: &Public) -> Result<Option<Raw>, anyhow::Error> {
        Ok(self.account_balances.get(account).map(|b| b.to_owned()))
    }

    async fn set_account_balance(&mut self, account: &Public, raw: &Raw) -> anyhow::Result<()> {
        self.account_balances
            .insert(account.to_owned(), raw.to_owned());
        Ok(())
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
