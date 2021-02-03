use rand::RngCore;

#[derive(Debug, Clone)]
pub struct Cookie([u8; Cookie::LENGTH]);

impl Cookie {
    pub const LENGTH: usize = 32;

    pub fn new() -> Self {
        let mut cookie = Cookie([0u8; Self::LENGTH]);
        rand::thread_rng().fill_bytes(&mut cookie.0);
        cookie
    }
}
