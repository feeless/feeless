use crate::cookie::Cookie;

pub struct NodeIdHandshakeQuery {}

impl NodeIdHandshakeQuery {
    pub const LENGTH: usize = Cookie::LENGTH;
}

pub struct NodeIdHandshakeResponse {}

impl NodeIdHandshakeResponse {
    pub const LENGTH: usize = Cookie::LENGTH;
}
