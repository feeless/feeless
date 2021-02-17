use crate::node::cookie::Cookie;
use crate::node::network::Network;
use crate::{BlockHash, FullBlock, Public, Raw};
use async_trait::async_trait;
pub use memory::MemoryState;
pub use sled_disk::SledDiskState;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::net::SocketAddr;

mod memory;
mod sled_disk;

pub type BoxedState = Box<dyn State + Send + Sync>;

/// State contains a state of the Nano block lattice 🥬.
#[async_trait]
pub trait State: Debug {
    fn network(&self) -> Network;

    async fn add_block(&mut self, account: &Public, full_block: &FullBlock) -> anyhow::Result<()>;

    async fn get_block_by_hash(&mut self, hash: &BlockHash) -> anyhow::Result<Option<FullBlock>>;

    /// Returns None if there is no opened account.
    async fn account_balance(&mut self, account: &Public) -> anyhow::Result<Option<Raw>>;

    async fn set_account_balance(&mut self, account: &Public, raw: &Raw) -> anyhow::Result<()>;

    async fn account_for_block_hash(
        &mut self,
        block_hash: &BlockHash,
    ) -> anyhow::Result<Option<Public>>;

    async fn set_cookie(&mut self, socket_addr: SocketAddr, cookie: Cookie) -> anyhow::Result<()>;

    async fn cookie_for_socket_addr(
        &self,
        socket_addr: &SocketAddr,
    ) -> anyhow::Result<Option<Cookie>>;
}
