use crate::encoding;

use crate::keys::public::Public;
use anyhow::anyhow;
use bitvec::prelude::*;
use regex::Regex;
use std::convert::TryFrom;

// nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7
// [   ][encoded public key                                ][chksum]
// [5  ][52                                                ][8     ] <-- Bytes

#[derive(Debug, PartialEq)]
pub struct Address(String);

impl Address {
    /// Length of a Nano address.
    pub const LEN: usize = 65; // 5 + 52 + 8

    /// Length of "nano_".
    pub const PREFIX_LEN: usize = 5;

    /// Length of the encoded public key.
    pub const ENCODED_PUBLIC_KEY_LEN: usize = 52;

    /// 4 bits of padding in the front of the public key when encoding.
    pub const ENCODED_PADDED_BITS: usize = 4;

    pub fn to_public(&self) -> Public {
        // We don't need to check the checksum because we assume if it's already stored, it's valid.
        self.extract_public_key().unwrap()
    }

    fn extract_public_key(&self) -> anyhow::Result<Public> {
        let public_key_part =
            &self.0[Self::PREFIX_LEN..(Self::PREFIX_LEN + Self::ENCODED_PUBLIC_KEY_LEN)];
        debug_assert_eq!(public_key_part.len(), Self::ENCODED_PUBLIC_KEY_LEN);

        let bits = encoding::decode_nano_base_32(&public_key_part)?;
        debug_assert_eq!(bits.len(), 8 * Public::LEN + Self::ENCODED_PADDED_BITS);

        // Remove padding
        // The to_owned() here is necessary to ensure the vec is aligned half way through the byte.
        // Otherwise it will essentially ignore the [ENCODED_PADDED_BITS..] offset.
        let bits: &BitVec<Msb0, u8> = &bits[Self::ENCODED_PADDED_BITS..].to_owned();

        let public_key_bytes: Vec<u8> = bits.to_owned().into_vec();
        debug_assert_eq!(public_key_bytes.len(), Public::LEN);

        Public::try_from(public_key_bytes.as_slice())
    }

    fn validate_checksum(&self, public: &Public) -> anyhow::Result<()> {
        let idx = Self::PREFIX_LEN + Self::ENCODED_PUBLIC_KEY_LEN;
        let checksum = &self.0[idx..];
        if public.checksum() != checksum {
            return Err(anyhow!("Invalid checksum"));
        }
        Ok(())
    }
}

impl From<&Public> for Address {
    /// Convert from a public key to an address.
    ///
    /// https://docs.nano.org/integration-guides/the-basics/#account-public-address
    fn from(public: &Public) -> Self {
        let mut s = String::with_capacity(Self::LEN);
        s.push_str("nano_");

        // Public key -> nano_base_32
        const PKP_LEN: usize = Address::ENCODED_PADDED_BITS + 8 * Public::LEN;
        const PKP_CAPACITY: usize = Address::ENCODED_PADDED_BITS + 8 * Public::LEN + 4; // Capacity rounded up to 8 bits.
        let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PKP_CAPACITY);
        let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; Self::ENCODED_PADDED_BITS];
        bits.extend_from_bitslice(&pad);
        bits.extend_from_raw_slice(&public.as_bytes());
        debug_assert_eq!(bits.capacity(), PKP_CAPACITY);
        debug_assert_eq!(bits.len(), PKP_LEN);
        let public_key_part = encoding::encode_nano_base_32(&bits);
        s.push_str(&public_key_part);

        // Public key -> blake2(5) -> nano_base_32
        let checksum = public.checksum();
        s.push_str(&checksum);

        debug_assert_eq!(s.len(), Self::LEN);
        debug_assert_eq!(s.capacity(), Self::LEN);
        Address(s)
    }
}

impl TryFrom<&str> for Address {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO: Lazy
        let re = Regex::new("^nano_[13][13456789abcdefghijkmnopqrstuwxyz]{59}$")
            .expect("Could not build regexp for nano address");
        if !re.is_match(value) {
            return Err(anyhow!("Not a valid nano address: {}", value));
        }

        let address = Address(value.into());
        let public = address.extract_public_key()?;
        address.validate_checksum(&public)?;

        Ok(address)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
