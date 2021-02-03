use crate::cookie::Cookie;
use feeless::Address;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// State contains a shared state between peer connections.
#[derive(Clone)]
pub struct State {
    cookies: Arc<RwLock<HashMap<SocketAddr, Cookie>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            cookies: Default::default(),
        }
    }
}
