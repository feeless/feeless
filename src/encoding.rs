use anyhow::anyhow;
use bitvec::prelude::*;
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        s.push_str(&format!("{:02X}", byte));
    }
    s
}

pub fn hex_formatter(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02X}", byte)?;
    }
    Ok(())
}

pub fn blake2b(size: usize, data: &[u8]) -> Box<[u8]> {
    let mut blake = VarBlake2b::new(size).expect("Output size was zero");
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
    use crate::Address;
    use std::convert::TryFrom;

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

    #[test]
    fn from_good_str() {
        let good_addresses = vec![
            "nano_3uaydiszyup5zwdt93dahp7mri1cwa5ncg9t4657yyn3o4i1pe8sfjbimbas",
            "nano_1qgkdadcbwn65sp95gr144fuc99tm5tn6gx9y8ow9bgaam6r5ixgtx19tw93",
            "nano_3power3gwb43rs7u9ky3rsjp6fojftejceexfkf845sfczyue4q3r1hfpr3o",
            "nano_1jtx5p8141zjtukz4msp1x93st7nh475f74odj8673qqm96xczmtcnanos1o",
            "nano_1ebq356ex7n5efth49o1p31r4fmuuoara5tmwduarg7b9jphyxsatr3ja6g8",
        ];
        for s in good_addresses {
            assert!(Address::from_str(s).is_ok());
        }
    }

    #[test]
    fn from_bad_checksum() {
        // These are the same as above with one character changed in the checksum section.
        let bad_checksums = vec![
            "nano_3uaydiszyup5zwdt93dahp7mri1cwa5ncg9t4657yyn3o4i1pe8sfjbimba1",
            "nano_1qgkdadcbwn65sp95gr144fuc99tm5tn6gx9y8ow9bgaam6r5ixgtx19tw23",
            "nano_3power3gwb43rs7u9ky3rsjp6fojftejceexfkf845sfczyue4q3r1hfp33o",
            "nano_1jtx5p8141zjtukz4msp1x93st7nh475f74odj8673qqm96xczmtcnan4s1o",
            "nano_1ebq356ex7n5efth49o1p31r4fmuuoara5tmwduarg7b9jphyxsatr35a6g8",
        ];
        for s in bad_checksums {
            assert!(Address::from_str(s).is_err());
        }
    }

    #[test]
    fn from_bad_str() {
        let bad_addresses = vec![
            "",
            "ABC",
            "01234567890123456789012345678901234567890123456789012345678901234",
            "nano_😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😊🥺😉😍😘😚😜😂😝😳",
            "nano_012345678901234567890123456789012345678901234567890123456789",
        ];

        for s in bad_addresses {
            let result = Address::from_str(s);
            dbg!(&result);
            assert!(result.is_err())
        }
    }
}
