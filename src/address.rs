use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use anyhow::anyhow;
use bitvec::prelude::*;

use crate::encoding;
use crate::encoding::blake2b;
use crate::public::{Public, PUBLIC_KEY_BYTES};
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use regex::Regex;

// nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7
// [   ][encoded public key                                ][chksum]
// [5  ][52                                                ][8     ] <-- Bytes

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
        let bits =
            encoding::decode_nano_base_32(&public_key_part).expect("Could not decode address");
        debug_assert_eq!(bits.len(), 8 * PUBLIC_KEY_LENGTH + 4);
        let bits: &BitVec<Msb0, u8> = &bits[ENCODED_PADDED_BITS..].to_owned(); // Remove padding
        let public_key_bytes: Vec<u8> = bits.to_owned().into_vec();
        debug_assert_eq!(public_key_bytes.len(), PUBLIC_KEY_LENGTH);

        // TODO: Check the checksum!!!!!!!!

        Public::try_from(public_key_bytes.as_slice())
            .expect("Could not create public key from address")
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

        // Try to convert to public key to check if the checksum is valid.
        address.to_public()?;

        Ok(address)
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
    fn from_good_str() {
        let good_addresses = vec![
            "nano_3uaydiszyup5zwdt93dahp7mri1cwa5ncg9t4657yyn3o4i1pe8sfjbimbas",
            "nano_1qgkdadcbwn65sp95gr144fuc99tm5tn6gx9y8ow9bgaam6r5ixgtx19tw93",
            "nano_3power3gwb43rs7u9ky3rsjp6fojftejceexfkf845sfczyue4q3r1hfpr3o",
            "nano_1jtx5p8141zjtukz4msp1x93st7nh475f74odj8673qqm96xczmtcnanos1o",
            "nano_1ebq356ex7n5efth49o1p31r4fmuuoara5tmwduarg7b9jphyxsatr3ja6g8",
        ];
        for s in good_addresses {
            assert!(Address::try_from(s).is_ok());
        }
    }

    #[test]
    fn from_bad_checksum() {
        let bad_checksums = vec![
            "nano_3uaydiszyup5zwdt93dahp7mri1cwa5ncg9t4657yyn3o4i1pe8sfjbimba1",
            "nano_1qgkdadcbwn65sp95gr144fuc99tm5tn6gx9y8ow9bgaam6r5ixgtx19tw23",
            "nano_3power3gwb43rs7u9ky3rsjp6fojftejceexfkf845sfczyue4q3r1hfp33o",
            "nano_1jtx5p8141zjtukz4msp1x93st7nh475f74odj8673qqm96xczmtcnan4s1o",
            "nano_1ebq356ex7n5efth49o1p31r4fmuuoara5tmwduarg7b9jphyxsatr35a6g8",
        ];
        for s in good_addresses {
            assert!(Address::try_from(s).is_err());
        }
    }

    #[test]
    fn from_bad_str() {
        let bad_addresses = vec![
            // Wrong length
            "",
            "ABC",
            // Doesn't start with nano_
            "01234567890123456789012345678901234567890123456789012345678901234",
            "😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😊🥺😉😍😘😚😜😂😝😳",
            // Incorrect checksum
            "nano_012345678901234567890123456789012345678901234567890123456789",
            "nano_😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😚😜😂😝😳😊🥺😉😍😘😊🥺😉😍😘😚😜😂😝😳",
        ];

        for s in bad_addresses {
            let result = Address::try_from(s);
            dbg!(&result);
            assert!(result.is_err())
        }
    }
}
