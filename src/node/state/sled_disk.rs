use std::net::SocketAddr;

use crate::node::cookie::Cookie;
use crate::node::network::Network;
use crate::node::state::State;
use crate::{Block, BlockHash, Public};
use async_trait::async_trait;
use std::convert::TryFrom;

/// Sled is an on disk key value pair.
#[derive(Clone, Debug)]
pub struct SledDiskState {
    network: Network,
    db: sled::Db,
    cookies: sled::Tree,
    peers: sled::Tree,
}

impl SledDiskState {
    pub fn new(network: Network) -> Self {
        let path = format!("{:?}.db", network).to_ascii_lowercase();
        let db: sled::Db =
            sled::open(&path).unwrap_or_else(|_| panic!("Could not open database: {}", &path));
        let cookies = db.open_tree("cookies").unwrap();
        let peers = db.open_tree("peers").unwrap();
        Self {
            network,
            db,
            cookies,
            peers,
        }
    }
}

#[async_trait]
impl State for SledDiskState {
    fn network(&self) -> Network {
        self.network
    }

    async fn add_block(&mut self, _account: &Public, _full_block: &Block) -> anyhow::Result<()> {
        unimplemented!()
    }

    async fn get_block_by_hash(&self, _hash: &BlockHash) -> anyhow::Result<Option<Block>> {
        unimplemented!()
    }

    async fn get_latest_block_hash_for_account(
        &self,
        _account: &Public,
    ) -> anyhow::Result<Option<BlockHash>> {
        unimplemented!()
    }

    async fn account_for_block_hash(
        &mut self,
        _block_hash: &BlockHash,
    ) -> Result<Option<Public>, anyhow::Error> {
        unimplemented!()
    }

    async fn set_cookie(&mut self, socket_addr: SocketAddr, cookie: Cookie) -> anyhow::Result<()> {
        self.cookies
            .insert(format!("{}", socket_addr), cookie.as_bytes())?;
        Ok(())
    }

    async fn cookie_for_socket_addr(
        &self,
        socket_addr: &SocketAddr,
    ) -> anyhow::Result<Option<Cookie>> {
        let maybe_cookie = self.cookies.get(format!("{}", socket_addr))?;
        Ok(match maybe_cookie.as_ref() {
            None => None,
            Some(c) => Some(Cookie::try_from(c.as_ref())?),
        })
    }
}
