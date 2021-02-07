use crate::cookie::Cookie;
use crate::header::Network;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// State contains a shared state between connections.
#[derive(Clone, Debug)]
pub struct State {
    network: Network,
    db: sled::Db,
    cookies: sled::Tree,
    peers: sled::Tree,
}

impl State {
    pub fn new(network: Network) -> Self {
        let path = format!("{:?}.db", network).to_ascii_lowercase();
        let db: sled::Db = sled::open(&path).expect(&format!("Could not open database: {}", &path));
        let cookies = db.open_tree("cookies").unwrap();
        let peers = db.open_tree("peers").unwrap();
        Self {
            network,
            db,
            cookies,
            peers,
        }
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub async fn set_cookie(
        &mut self,
        socket_addr: SocketAddr,
        cookie: Cookie,
    ) -> anyhow::Result<()> {
        self.cookies
            .insert(format!("{}", socket_addr), cookie.as_bytes())?;
        Ok(())
    }

    pub async fn cookie_for_socket_addr(
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
