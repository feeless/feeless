use anyhow::anyhow;
use bitvec::prelude::*;
use blake2::digest::{Update, VariableOutput};
use blake2::{Digest, VarBlake2b};

pub fn fmt_hex(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02X}", byte)?;
    }
    Ok(())
}

pub fn blake2b(size: usize, data: &[u8]) -> Box<[u8]> {
    let mut blake = VarBlake2b::new(size).expect("output size was zero");
    blake.update(&data);
    blake.finalize_boxed()
}

const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
const ENCODING_BITS: usize = 5;

pub fn encode_nano_base_32(bits: &BitSlice<Msb0, u8>) -> String {
    debug_assert_eq!(
        bits.len() % ENCODING_BITS,
        0,
        "BitSlice must be divisible by 5"
    );
    let mut s = String::new(); // TODO: with_capacity
    let alphabet: Vec<char> = ALPHABET.chars().collect(); // TODO: lazy
    for idx in (0..bits.len()).step_by(ENCODING_BITS) {
        let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + ENCODING_BITS];
        let value: u8 = chunk.load_be();
        let char = alphabet[value as usize];
        s.push(char);
    }
    s
}

pub fn decode_nano_base_32(s: &str) -> anyhow::Result<BitVec<Msb0, u8>> {
    let mut bits: BitVec<Msb0, u8> = BitVec::new(); // TODO: with_capacity
    for char in s.chars() {
        let value = ALPHABET
            .find(char) // TODO: performance
            .ok_or_else(|| anyhow!("Unknown character found while decoding: {}", char))?;
        let value = value as u8;
        let char_bits: &BitSlice<Msb0, u8> = value.view_bits();
        bits.extend_from_bitslice(&char_bits[(8 - ENCODING_BITS)..8]);
    }

    Ok(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let bits: BitVec<Msb0, u8> =
            bitvec![Msb0, u8; 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
        let encoded = encode_nano_base_32(&bits);
        let decoded = decode_nano_base_32(&encoded).unwrap();
        assert_eq!(bits, decoded);
    }

    #[test]
    fn decode_in_order() {
        let decoded = decode_nano_base_32(ALPHABET).unwrap();
        assert_eq!(decoded.len(), ENCODING_BITS * ALPHABET.len());
        for d in 0..(ALPHABET.len() / ENCODING_BITS) {
            let idx = d * ENCODING_BITS;
            let chunk: &BitSlice<Msb0, u8> = &decoded[idx..(idx + ENCODING_BITS)];
            let value: u8 = chunk.load_be();
            assert_eq!(d as u8, value);
        }
    }
}
