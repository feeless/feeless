use crate::Error;
use bitvec::prelude::*;
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        s.push_str(&format!("{:02X}", byte));
    }
    s
}

pub fn to_hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

pub fn hex_formatter(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02X}", byte)?;
    }
    Ok(())
}

pub fn hex_formatter_lower(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02x}", byte)?;
    }
    Ok(())
}

pub fn deserialize_from_str<'de, T, D>(
    deserializer: D,
) -> Result<T, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(T::from_str(s).map_err(serde::de::Error::custom)?)
}

pub fn deserialize_from_string<'de, T, D>(
    deserializer: D,
) -> Result<T, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(T::from_str(s.as_str()).map_err(serde::de::Error::custom)?)
}

pub fn blake2b(size: usize, data: &[u8]) -> Box<[u8]> {
    let mut blake = VarBlake2b::new(size).expect("Output size was zero");
    blake.update(&data);
    blake.finalize_boxed()
}

/// Use this instead of [blake2b] to probably prevent an allocation.
pub fn blake2b_callback(size: usize, data: &[u8], f: impl FnOnce(&[u8])) {
    let mut blake = VarBlake2b::new(size).expect("Output size was zero");
    blake.update(&data);
    blake.finalize_variable(f)
}

pub(crate) const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
static ALPHABET_VEC: Lazy<Vec<char>> = Lazy::new(|| ALPHABET.chars().collect());
const ENCODING_BITS: usize = 5;

pub fn encode_nano_base_32(bits: &BitSlice<Msb0, u8>) -> String {
    debug_assert_eq!(
        bits.len() % ENCODING_BITS,
        0,
        "BitSlice must be divisible by 5"
    );
    let mut s = String::new(); // TODO: with_capacity
    for idx in (0..bits.len()).step_by(ENCODING_BITS) {
        let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + ENCODING_BITS];
        let value: u8 = chunk.load_be();
        let char = ALPHABET_VEC[value as usize];
        s.push(char);
    }
    s
}

pub fn decode_nano_base_32(s: &str) -> Result<BitVec<Msb0, u8>, Error> {
    let mut bits: BitVec<Msb0, u8> = BitVec::new(); // TODO: with_capacity
    for char in s.chars() {
        let value = ALPHABET
            .find(char) // TODO: performance
            .ok_or_else(|| Error::DecodingError(char))?;
        let value = value as u8;
        let char_bits: &BitSlice<Msb0, u8> = value.view_bits();
        bits.extend_from_bitslice(&char_bits[(8 - ENCODING_BITS)..8]);
    }

    Ok(bits)
}

/// This macro relies on the `struct` to be a newtype containing a slice of `[u8; $struct::LEN]`.
///
/// It adds:
/// * serde implementations to (de)serialize hex strings.
/// * `pub fn as_bytes(&self) -> &[u8]`
/// * `pub fn as_hex(&self) -> String`
/// * `TryFrom<&[u8]>` implementation.
/// * [FromStr] implementation, which parses hex into its type.
/// * [Debug] implementation, which displays as StructName(H3XSTR1NG), e.g. Work(A1B2C3).
/// * [UpperHex] and [LowerHex] implementations.
///
/// Display implementation is not implemented for any user customization.
#[macro_export]
macro_rules! hexify {
    ($struct:ident, $description:expr) => {
        impl $struct {
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            pub fn as_hex(&self) -> String {
                crate::encoding::to_hex(&self.0)
            }

            pub fn as_hex_lower(&self) -> String {
                crate::encoding::to_hex_lower(&self.0)
            }
        }

        impl ::std::str::FromStr for $struct {
            type Err = crate::Error;

            fn from_str(s: &str) -> crate::Result<Self> {
                use ::std::convert::TryFrom;

                crate::expect_len(s.len(), Self::LEN * 2, $description)?;
                let vec = hex::decode(s.as_bytes()).map_err(|e| crate::Error::FromHexError {
                    msg: String::from($description),
                    source: e,
                })?;
                let bytes = vec.as_slice();
                let x = <[u8; Self::LEN]>::try_from(bytes)?;
                Ok(Self(x))
            }
        }

        impl ::std::fmt::Display for $struct {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_hex())
            }
        }

        impl ::std::fmt::Debug for $struct {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(
                    f,
                    "{}({})",
                    stringify!($struct),
                    crate::encoding::to_hex(self.0.as_ref()),
                )
            }
        }

        impl ::std::convert::TryFrom<&[u8]> for $struct {
            type Error = crate::Error;

            fn try_from(v: &[u8]) -> crate::Result<Self> {
                Ok(Self(<[u8; Self::LEN]>::try_from(v)?))
            }
        }

        impl ::std::fmt::UpperHex for $struct {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_hex())
            }
        }

        impl ::std::fmt::LowerHex for $struct {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_hex_lower())
            }
        }

        impl serde::Serialize for $struct {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_hex().as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $struct {
            fn deserialize<D>(
                deserializer: D,
            ) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use ::std::str::FromStr;
                let s: String = serde::Deserialize::deserialize(deserializer)?;
                Ok(Self::from_str(&s).map_err(serde::de::Error::custom)?)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Address;
    use std::str::FromStr;

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
