use crate::blocks::{Block, BlockHash};

use crate::node::cookie::Cookie;
use crate::Public;
use async_trait::async_trait;
pub use memory::MemoryState;
pub use sled_disk::SledDiskState;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
mod memory;
mod sled_disk;

pub type DynState = dyn State + Send + Sync;
pub type ArcState = Arc<Mutex<DynState>>;

/// State contains a state of the Nano block lattice 🥬.
#[async_trait]
pub trait State: Debug + Sync + Send + 'static {
    async fn add_block(&mut self, block: &Block) -> anyhow::Result<()>;

    async fn get_block_by_hash(&self, hash: &BlockHash) -> anyhow::Result<Option<Block>>;

    async fn get_latest_block_hash_for_account(
        &self,
        account: &Public,
    ) -> anyhow::Result<Option<BlockHash>>;

    async fn account_for_block_hash(
        &mut self,
        block_hash: &BlockHash,
    ) -> anyhow::Result<Option<Public>>;

    async fn add_vote(&mut self, hash: &BlockHash, representative: &Public) -> anyhow::Result<()>;

    async fn set_cookie(&mut self, socket_addr: SocketAddr, cookie: Cookie) -> anyhow::Result<()>;

    async fn cookie_for_socket_addr(
        &self,
        socket_addr: &SocketAddr,
    ) -> anyhow::Result<Option<Cookie>>;
}
