use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use anyhow::anyhow;
use bitvec::prelude::*;
use bitvec::view::AsBits;

use crate::encoding;
use crate::encoding::blake2b;
use crate::public::{Public, PUBLIC_KEY_BYTES};
use ed25519_dalek::PUBLIC_KEY_LENGTH;

// nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7
// [   ][encoded public key                                ][chksum]
// [5B ][52B                                               ][8B    ]

/// Length of "nano_".
const PREFIX_LEN: usize = 5;

/// Length of the encoded public key.
const ENCODED_PUBLIC_KEY_LEN: usize = 52;

/// Length of the encoded checksum.
const ENCODED_CHECKSUM_LENGTH: usize = 8;

/// Length of a Nano address.
const ADDRESS_STRING_LENGTH: usize = 65; // 5 + 52 + 8

const ENCODED_PADDED_BITS: usize = 4;

const DECODED_CHECKSUM_LENGTH: usize = 5;

#[derive(Debug, PartialEq)]
pub struct Address(String);

impl Address {
    pub fn to_public(&self) -> Public {
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

        let public_key_part = &self.0[PREFIX_LEN..(PREFIX_LEN + ENCODED_PUBLIC_KEY_LEN)];
        debug_assert_eq!(public_key_part.len(), ENCODED_PUBLIC_KEY_LEN);
        let decoded_public_key_with_padding =
            encoding::decode_nano_base_32(&public_key_part).expect("Could not decode address");
        debug_assert_eq!(
            decoded_public_key_with_padding.len(),
            8 * PUBLIC_KEY_LENGTH + 4
        );

        // Without padding
        let decoded_public_key: &BitVec<Msb0, u8> =
            &BitVec::from_bitslice(&decoded_public_key_with_padding[ENCODED_PADDED_BITS..]);
        debug_assert_eq!(decoded_public_key.len(), 8 * PUBLIC_KEY_LENGTH);

        dbg!(&decoded_public_key);

        // let public_key_bytes = decoded_public_key.domain().region().unwrap();
        let public_key_bytes: &[u8] = decoded_public_key.as_raw_slice();
        dbg!(&public_key_bytes);
        debug_assert_eq!(public_key_bytes.len(), PUBLIC_KEY_LENGTH);

        Public::try_from(public_key_bytes).expect("Could not create public key from address")

        // todo!()
    }
}

impl From<&Public> for Address {
    /// https://docs.nano.org/integration-guides/the-basics/#account-public-address
    fn from(public: &Public) -> Self {
        let mut address = String::with_capacity(ADDRESS_STRING_LENGTH);
        address.push_str("nano_");

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
        address.push_str(&public_key_part);

        // Public key -> blake2(5) -> nano_base_32
        let result = blake2b(DECODED_CHECKSUM_LENGTH, &public.as_bytes());
        let bits = BitVec::from_iter(result.iter().rev());
        let checksum_part = encoding::encode_nano_base_32(&bits);
        address.push_str(&checksum_part);

        debug_assert_eq!(address.len(), ADDRESS_STRING_LENGTH);
        debug_assert_eq!(address.capacity(), ADDRESS_STRING_LENGTH);
        Address(address)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::{decode_nano_base_32, encode_nano_base_32};

    use super::*;

    #[test]
    fn from_str() {
        let bad_fixtures = vec![
            // Wrong length
            "",
            // Doesn't start with nano_
            "01234567890123456789012345678901234567890123456789012345678901234",
            // Incrrect checksum
            "nano_012345678901234567890123456789012345678901234567890123456789",
        ];

        for s in bad_fixtures {
            assert!(Public::try_from(s.as_bytes()).is_err())
        }
    }
}
