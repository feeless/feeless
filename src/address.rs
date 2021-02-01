use crate::encoding;

use crate::public::{Public, PUBLIC_KEY_BYTES};
use anyhow::anyhow;
use bitvec::prelude::*;
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use regex::Regex;
use std::convert::TryFrom;

// nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7
// [   ][encoded public key                                ][chksum]
// [5  ][52                                                ][8     ] <-- Bytes

/// Length of "nano_".
const PREFIX_LEN: usize = 5;

/// Length of the encoded public key.
const ENCODED_PUBLIC_KEY_LEN: usize = 52;

/// Length of a Nano address.
const ADDRESS_STRING_LENGTH: usize = 65; // 5 + 52 + 8

const ENCODED_PADDED_BITS: usize = 4;

#[derive(Debug, PartialEq)]
pub struct Address(String);

impl Address {
    pub fn to_public(&self) -> Public {
        // We don't need to check the checksum because we assume if it's already stored, it's valid.
        self.extract_public_key().unwrap()
    }

    fn sanity_check(&self) {
        debug_assert_eq!(
            self.0.len(),
            ADDRESS_STRING_LENGTH,
            "Address length is {} and needs to be {}",
            self.0.len(),
            ADDRESS_STRING_LENGTH
        );
        debug_assert!(
            self.0.starts_with("nano_"),
            "Address needs to start with nano_"
        );
    }

    fn extract_public_key(&self) -> anyhow::Result<Public> {
        self.sanity_check();

        let public_key_part = &self.0[PREFIX_LEN..(PREFIX_LEN + ENCODED_PUBLIC_KEY_LEN)];
        debug_assert_eq!(public_key_part.len(), ENCODED_PUBLIC_KEY_LEN);

        let bits = encoding::decode_nano_base_32(&public_key_part)?;
        debug_assert_eq!(bits.len(), 8 * PUBLIC_KEY_LENGTH + 4);

        let bits: &BitVec<Msb0, u8> = &bits[ENCODED_PADDED_BITS..].to_owned(); // Remove padding
        let public_key_bytes: Vec<u8> = bits.to_owned().into_vec();
        debug_assert_eq!(public_key_bytes.len(), PUBLIC_KEY_LENGTH);

        Ok(Public::try_from(public_key_bytes.as_slice())?)
    }

    fn validate_checksum(&self, public: &Public) -> anyhow::Result<()> {
        let idx = PREFIX_LEN + ENCODED_PUBLIC_KEY_LEN;
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
        let mut s = String::with_capacity(ADDRESS_STRING_LENGTH);
        s.push_str("nano_");

        // Public key -> nano_base_32
        const PKP_LEN: usize = ENCODED_PADDED_BITS + 8 * PUBLIC_KEY_BYTES;
        const PKP_CAPACITY: usize = ENCODED_PADDED_BITS + 8 * PUBLIC_KEY_BYTES + 4; // Capacity rounded up to 8 bits.
        let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PKP_CAPACITY);
        let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; ENCODED_PADDED_BITS];
        bits.extend_from_bitslice(&pad);
        bits.extend_from_raw_slice(&public.as_bytes());
        debug_assert_eq!(bits.capacity(), PKP_CAPACITY);
        debug_assert_eq!(bits.len(), PKP_LEN);
        let public_key_part = encoding::encode_nano_base_32(&bits);
        s.push_str(&public_key_part);

        // Public key -> blake2(5) -> nano_base_32
        let checksum = public.checksum();
        s.push_str(&checksum);

        debug_assert_eq!(s.len(), ADDRESS_STRING_LENGTH);
        debug_assert_eq!(s.capacity(), ADDRESS_STRING_LENGTH);
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
