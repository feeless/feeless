use crate::node::cookie::Cookie;
use crate::node::network::Network;
use crate::{BlockHash, FullBlock};
use async_trait::async_trait;
pub use memory::MemoryState;
pub use sled_disk::SledDiskState;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::net::SocketAddr;

mod memory;
mod sled_disk;

pub type BoxedState = Box<dyn State + Send + Sync>;

/// State contains a shared state between connections.
#[async_trait]
pub trait State: Debug {
    fn network(&self) -> Network;

    async fn add_block(&mut self, full_block: &FullBlock) -> anyhow::Result<()>;
    async fn get_block_by_hash(&mut self, hash: &BlockHash) -> anyhow::Result<Option<FullBlock>>;

    async fn set_cookie(&mut self, socket_addr: SocketAddr, cookie: Cookie) -> anyhow::Result<()>;
    async fn cookie_for_socket_addr(
        &self,
        socket_addr: &SocketAddr,
    ) -> anyhow::Result<Option<Cookie>>;
}
