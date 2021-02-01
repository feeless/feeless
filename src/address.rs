use bitvec::prelude::*;
use crate::key::Public;

const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
const PADDED_BITS: usize = 4;

pub struct Address(String);

impl From<&Public> for Address {
    fn from(public: &Public) -> Self {
        let alphabet: Vec<char> = ALPHABET.chars().collect(); // TODO: lazy?
        let mut address = String::with_capacity(60);
        let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PADDED_BITS + 8 * 32);
        let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; PADDED_BITS];
        bits.extend_from_bitslice(&pad);
        bits.extend_from_raw_slice(&public.as_bytes());
        dbg!(&bits);
        for idx in (0..bits.len()).step_by(5) {
            let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + 5];
            let value: u8 = chunk.load_be();
            let char = alphabet[value as usize];
            dbg!(chunk, value);
            address.push(char);
        }

        dbg!(address);

        // TODO: Not sure about this unwrap?
        // let bits = BitSlice::<Msb0, _>::from_slice(&public.as_bytes()).unwrap();
        // let bits = padded + bits;
        // let first = &bits[0..5];
        // dbg!(first);
        todo!()
    }
}

