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
            cookies: Default::default(),
        }
    }

    pub fn network(&self) -> Network {
        self.network
    }
}
