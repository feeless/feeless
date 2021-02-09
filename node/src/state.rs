use crate::cookie::Cookie;
use crate::header::Network;

use async_trait::async_trait;

use std::convert::TryFrom;
use std::fmt::Debug;
use std::net::SocketAddr;

pub type BoxedState = Box<dyn State + Send + Sync>;

#[async_trait]
pub trait State: Debug {
    fn network(&self) -> Network;

    async fn set_cookie(&mut self, socket_addr: SocketAddr, cookie: Cookie) -> anyhow::Result<()>;

    async fn cookie_for_socket_addr(
        &self,
        socket_addr: &SocketAddr,
    ) -> anyhow::Result<Option<Cookie>>;
}

/// State contains a shared state between connections.
#[derive(Clone, Debug)]
pub struct SledState {
    network: Network,
    db: sled::Db,
    cookies: sled::Tree,
    peers: sled::Tree,
}

impl SledState {
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
impl State for SledState {
    fn network(&self) -> Network {
        self.network
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

#[derive(Debug)]
pub struct TestState {
    network: Network,
}

impl TestState {
    pub fn new(network: Network) -> Self {
        Self { network }
    }
}

#[async_trait]
impl State for TestState {
    fn network(&self) -> Network {
        self.network
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
