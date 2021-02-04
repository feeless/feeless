use crate::cookie::Cookie;
use crate::header::Network;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// State contains a shared state between connections.
#[derive(Clone)]
pub struct State {
    network: Network,
    cookies: Arc<RwLock<HashMap<SocketAddr, Cookie>>>,
}

impl State {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            cookies: Arc::new(RwLock::new(HashMap::with_capacity(1000))),
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
        let mut cookies = &mut *self.cookies.write().await;
        cookies.insert(socket_addr, cookie);
        Ok(())
    }

    pub async fn cookie_for_socket_addr(&self, socket_addr: &SocketAddr) -> anyhow::Result<Cookie> {
        let cookies = self.cookies.read().await;
        let cookie = (*cookies).get(socket_addr).unwrap(); // TODO: handle missing entry
        Ok(cookie.clone())
    }
}
