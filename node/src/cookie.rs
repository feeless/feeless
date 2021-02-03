use anyhow::anyhow;
use rand::RngCore;
use std::convert::TryFrom;

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

impl TryFrom<&[u8]> for Cookie {
    type Error = anyhow::Error;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if v.len() != Self::LENGTH {
            return Err(anyhow!(
                "Incorrect cookie length. Got: {} Expecting: {}",
                v.len(),
                Self::LENGTH,
            ));
        }

        let mut cookie = Self::new();
        cookie.0.copy_from_slice(v);
        Ok(cookie)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let mut c1 = Cookie::new();
        c1.0[0] = 0xff;
        c1.0[31] = 0x00;
        let c2 = Cookie::try_from(c1.0.as_ref()).unwrap();
        assert_eq!(c1.0[0], c2.0[0]);
        assert_eq!(c1.0[31], c2.0[31]);
    }
}
