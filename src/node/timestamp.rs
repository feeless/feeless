use crate::len_err_msg;
use anyhow::Context;
use bitvec::prelude::*;
use bitvec::view::AsBits;
use once_cell::sync::Lazy;
use std::convert::TryFrom;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const TIME_BITS: usize = 44;
const COUNT_BITS: usize = 20;

static COUNT_LOCK: Lazy<Mutex<IncrementalTimestampState>> =
    Lazy::new(|| Mutex::new(IncrementalTimestampState { ms: 0, count: 0 }));

#[derive(Eq, PartialEq, Debug)]
pub struct IncrementalTimestampState {
    pub ms: u64,
    pub count: u64,
}

impl IncrementalTimestampState {
    pub fn from_u64(s: u64) -> Self {
        let bits: &BitSlice<Msb0, u64> = s.view_bits();
        Self {
            ms: bits[0..TIME_BITS].to_owned().load(),
            count: bits[TIME_BITS..].to_owned().load(),
        }
    }

    fn to_u64(&self) -> u64 {
        let mut bits: BitVec<Msb0, u64> = BitVec::with_capacity(64);
        let ms_bits = &self.ms.view_bits::<Msb0>()[64 - TIME_BITS..];
        bits.extend_from_bitslice(&ms_bits);
        debug_assert_eq!(bits.len(), TIME_BITS);

        let count_bits = &self.count.view_bits::<Msb0>()[64 - COUNT_BITS..];
        bits.extend_from_bitslice(&count_bits);
        debug_assert_eq!(bits.len(), COUNT_BITS + TIME_BITS);

        bits.load_be()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IncrementalTimestamp(u64);

impl IncrementalTimestamp {
    pub const LEN: usize = 8;

    pub fn now() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards"); // TODO: Handle this nicely!
        let timestamp = since_the_epoch.as_millis() as u64;

        let mut state = COUNT_LOCK.lock().unwrap();
        if state.ms != timestamp {
            dbg!("ts changed");
            state.count = 0;
            state.ms = timestamp;
        } else {
            state.count += 1;
            dbg!("ts same, count now", state.count);
        }

        Self(state.to_u64())
    }

    pub fn from_u64(n: u64) -> Self {
        IncrementalTimestamp(n)
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        self.0.to_be_bytes()
    }

    pub fn get(&self) -> IncrementalTimestampState {
        IncrementalTimestampState::from_u64(self.0)
    }
}

impl TryFrom<&[u8]> for IncrementalTimestamp {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let fixed = <[u8; Self::LEN]>::try_from(value)
            .with_context(|| len_err_msg(value.len(), Self::LEN, "IncrementalTimestamp"))?;

        let num = u64::from_be_bytes(fixed);
        Ok(IncrementalTimestamp::from_u64(num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_to_u64() {
        let state = IncrementalTimestampState {
            ms: 1614200740266,
            count: 300,
        };
        let a = state.to_u64();
        let b = IncrementalTimestampState::from_u64(a);
        assert_eq!(state, b);
    }

    #[test]
    fn encoding() {
        IncrementalTimestamp::now();
        IncrementalTimestamp::now();
        let it = IncrementalTimestamp::now();
        let bytes = it.to_bytes();
        dbg!(&bytes);
        let back = IncrementalTimestamp::try_from(bytes.as_ref()).unwrap();
        assert_eq!(it, back);
        assert_eq!(back.get().count, 2);
    }
}
