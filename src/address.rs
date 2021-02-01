use bitvec::prelude::*;
use crate::key::Public;
use std::iter::FromIterator;
use bitvec::macros::internal::core::fmt::Formatter;
use crate::blake2b;

const ADDRESS_LENGTH: usize = 65;
const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
const PADDED_BITS: usize = 4;

pub struct Address(String);

fn encode_nano_base_32(bits: &BitSlice<Msb0, u8>) -> String {
    let mut s = String::new(); // TODO: with_capacity
    let alphabet: Vec<char> = ALPHABET.chars().collect(); // TODO: lazy
    for idx in (0..bits.len()).step_by(5) {
        let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + 5];
        let value: u8 = chunk.load_be();
        let char = alphabet[value as usize];
        dbg!(chunk, value);
        s.push(char);
    }
    s
}

impl From<&Public> for Address {
    /// https://docs.nano.org/integration-guides/the-basics/#account-public-address
    fn from(public: &Public) -> Self {
        let mut address = String::with_capacity(ADDRESS_LENGTH);
        address.push_str("nano_");

        // Public key -> nano_base_32
        let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PADDED_BITS + 8 * 32);
        let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; PADDED_BITS];
        bits.extend_from_bitslice(&pad);
        bits.extend_from_raw_slice(&public.as_bytes());
        let public_key_part = encode_nano_base_32(&bits);
        address.push_str(&public_key_part);
        dbg!(&public_key_part);

        // Public key -> blake2(5) -> nano_base_32
        let result = blake2b(5, &public.as_bytes());
        let bits = BitVec::from_iter(result.iter().rev());
        let checksum_part = encode_nano_base_32(&bits);
        dbg!(&checksum_part);
        address.push_str(&checksum_part);

        debug_assert_eq!(address.len(), ADDRESS_LENGTH);
        debug_assert_eq!(address.capacity(), ADDRESS_LENGTH);

        Address(address)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}