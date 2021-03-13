use crate::blocks::{Block, BlockHash};
use crate::network::Network;
use crate::node::cookie::Cookie;
use crate::node::state::State;
use crate::Public;
use anyhow::Context;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct MemoryState {
    network: Network,
    cookies: HashMap<SocketAddr, Cookie>,
    blocks: HashMap<BlockHash, Block>,
    block_hash_to_account: HashMap<BlockHash, Public>,
    latest_block_hash: HashMap<Public, BlockHash>,
    votes: HashMap<BlockHash, HashSet<Public>>,
}

impl MemoryState {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            cookies: HashMap::new(),
            blocks: HashMap::new(),
            block_hash_to_account: HashMap::new(),
            latest_block_hash: HashMap::new(),
            votes: HashMap::new(),
        }
    }
}

#[async_trait]
impl State for MemoryState {
    async fn add_block(&mut self, block: &Block) -> anyhow::Result<()> {
        self.blocks.insert(
            block.hash().context("Add block")?.to_owned(),
            block.to_owned(),
        );
        self.block_hash_to_account
            .insert(block.hash()?.to_owned(), block.account().to_owned());
        self.latest_block_hash
            .insert(block.account().to_owned(), block.hash()?.to_owned());
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

    async fn add_vote(&mut self, hash: &BlockHash, representative: &Public) -> anyhow::Result<()> {
        let entry = self
            .votes
            .entry(hash.to_owned())
            .or_insert_with(|| HashSet::new());
        entry.insert(representative.to_owned());

        // dbg!(&self
        //     .votes
        //     .iter()
        //     .map(|v| format!("{} {}", v.0, v.1.len()))
        //     .collect::<Vec<_>>());

        // dbg!(&self.votes);

        // println!("XXXXXX {:?} {:?}", hash, representative);

        Ok(())
    }

    async fn set_cookie(
        &mut self,
        socket_addr: SocketAddr,
        cookie: Cookie,
    ) -> Result<(), anyhow::Error> {
        self.cookies.insert(socket_addr, cookie);
        Ok(())
    }

    async fn cookie_for_socket_addr(
        &self,
        socket_addr: &SocketAddr,
    ) -> Result<Option<Cookie>, anyhow::Error> {
        Ok(self.cookies.get(&socket_addr).map(|c| c.to_owned()))
    }
}
