use crate::state::State;
use crate::wire::Wire;
use rand::RngCore;

use feeless::expect_len;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Cookie([u8; Cookie::LEN]);

impl Cookie {
    pub const LEN: usize = 32;

    pub fn random() -> Self {
        let mut cookie = Cookie([0u8; Self::LEN]);
        rand::thread_rng().fill_bytes(&mut cookie.0);
        cookie
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Wire for Cookie {
    fn serialize(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }

    fn deserialize(_: &State, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Cookie::try_from(data)?)
    }

    fn len() -> usize {
        Cookie::LEN
    }
}

impl TryFrom<&[u8]> for Cookie {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Cookie")?;

        // TODO: Self::zero()
        let mut cookie = Self::random();
        cookie.0.copy_from_slice(value);
        Ok(cookie)
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
