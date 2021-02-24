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

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct IncrementalTimestamp {
    pub ms: u64,
    pub count: u64,
}

impl IncrementalTimestamp {
    pub const LEN: usize = 8;

    pub fn new() -> Self {
        Self {
            ms: Self::now(),
            count: 0,
        }
    }

    pub fn next(&mut self) {
        let timestamp = Self::now();
        if self.ms != timestamp {
            self.count = 0;
            self.ms = timestamp;
        } else {
            self.count += 1;
        }
    }

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

    fn now() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards"); // TODO: Handle this nicely!
        since_the_epoch.as_millis() as u64
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        self.to_u64().to_be_bytes()
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
        let state = IncrementalTimestamp {
            ms: 1614200740266,
            count: 300,
        };
        let a = state.to_u64();
        let b = IncrementalTimestamp::from_u64(a);
        assert_eq!(state, b);
    }

    #[test]
    fn encoding() {
        let mut it = IncrementalTimestamp::new();
        it.next();
        it.next();

        let bytes = it.to_bytes();
        dbg!(&bytes);
        let back = IncrementalTimestamp::try_from(bytes.as_ref()).unwrap();
        assert_eq!(it, back);
        assert_eq!(back.count, 2);
    }
}
