use crate::cookie::Cookie;
use feeless::Address;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct State {
    cookies: HashMap<SocketAddr, Cookie>,
}

impl State {
    pub fn new() -> Self {
        Self {
            cookies: Default::default(),
        }
    }
}
