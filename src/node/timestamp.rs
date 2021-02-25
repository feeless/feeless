use crate::len_err_msg;
use anyhow::Context;
use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Timestamp(u64);

impl Timestamp {
    pub const LEN: usize = 8;

    pub fn now() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards"); // TODO: Handle this nicely!
        Self(since_the_epoch.as_millis() as u64)
    }

    pub fn from_u64(s: u64) -> Self {
        Self(s)
    }

    fn to_u64(&self) -> u64 {
        self.0
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        self.0.to_le_bytes()
    }
}

impl TryFrom<&[u8]> for Timestamp {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        dbg!(&value);
        let fixed = <[u8; Self::LEN]>::try_from(value)
            .with_context(|| len_err_msg(value.len(), Self::LEN, "IncrementalTimestamp"))?;

        let num = u64::from_le_bytes(fixed);
        Ok(Timestamp::from_u64(num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_to_u64() {
        let state = Timestamp::from_u64(1614200740266);
        let a = state.to_u64();
        let b = Timestamp::from_u64(a);
        assert_eq!(state, b);
    }

    #[test]
    fn encoding() {
        let mut it = Timestamp::now();
        let bytes = it.to_bytes();
        dbg!(&bytes);
        let back = Timestamp::try_from(bytes.as_ref()).unwrap();
        assert_eq!(it, back);
    }
}
