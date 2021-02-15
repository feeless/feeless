use std::net::SocketAddr;

use crate::node::cookie::Cookie;
use crate::node::network::Network;
use crate::node::state::State;
use crate::{BlockHash, FullBlock};
use async_trait::async_trait;

#[derive(Debug)]
pub struct MemoryState {
    network: Network,
}

impl MemoryState {
    pub fn new(network: Network) -> Self {
        Self { network }
    }
}

#[async_trait]
impl State for MemoryState {
    fn network(&self) -> Network {
        self.network
    }

    async fn add_block(&mut self, full_block: &FullBlock) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    async fn get_block_by_hash(
        &mut self,
        hash: &BlockHash,
    ) -> Result<Option<FullBlock>, anyhow::Error> {
        unimplemented!()
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
