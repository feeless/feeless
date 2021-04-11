use crate::hexify;
use crate::node::header::Header;
use crate::node::wire::Wire;
use anyhow::anyhow;
use rand::RngCore;
use std::convert::TryFrom;

#[derive(Clone)]
#[repr(C)]
pub struct Cookie([u8; Cookie::LEN]);

hexify!(Cookie, "cookie");

impl Cookie {
    pub const LEN: usize = 32;

    pub fn random() -> Self {
        let mut cookie = Cookie([0u8; Self::LEN]);
        rand::thread_rng().fill_bytes(&mut cookie.0);
        cookie
    }
}

impl Wire for Cookie {
    fn serialize(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }

    fn deserialize(_: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // TODO: thiserror
        Cookie::try_from(data).map_err(|e| anyhow!("Deserializing cookie {:?}", e))
    }

    fn len(_header: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Cookie::LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let mut c1 = Cookie::random();
        c1.0[0] = 0xff;
        c1.0[31] = 0x00;
        let c2 = Cookie::try_from(c1.0.as_ref()).unwrap();
        assert_eq!(c1.0[0], c2.0[0]);
        assert_eq!(c1.0[31], c2.0[31]);
    }
}
