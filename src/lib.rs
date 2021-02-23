#![forbid(unsafe_code)]

use anyhow::anyhow;

pub use blocks::link::Link;
pub use blocks::Block;
pub use blocks::BlockHash;
pub use encoding::{hex_formatter, to_hex};
pub use keys::address::Address;
pub use keys::phrase::{Language, MnemonicType, Phrase};
pub use keys::private::Private;
pub use keys::public::Public;
pub use keys::seed::Seed;
pub use pow::work::Work;
pub use raw::Raw;
pub use signature::Signature;

mod blocks;
mod bytes;
pub mod encoding;
mod keys;
#[cfg(feature = "node")]
pub mod node;
#[cfg(feature = "pcap")]
pub mod pcap;
mod pow;
mod pretty;
mod raw;
mod signature;

pub const DEFAULT_PORT: u16 = 7075;

pub fn expect_len(got_len: usize, expected_len: usize, msg: &str) -> anyhow::Result<()> {
    if got_len != expected_len {
        return Err(anyhow!(
            "{} is the wrong length: got: {} expected: {}",
            msg,
            got_len,
            expected_len,
        ));
    }
    Ok(())
}

pub fn len_err_msg(got_len: usize, expected_len: usize, msg: &str) -> String {
    format!(
        "{} is the wrong length: got: {} expected: {}",
        msg, got_len, expected_len,
    )
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;

    #[test]
    fn conversions() {
        let seed =
            Seed::try_from("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        let private: Private = seed.derive(0);
        assert_eq!(
            private.to_string(),
            "9F0E444C69F77A49BD0BE89DB92C38FE713E0963165CCA12FAF5712D7657120F"
        );

        let public = private.to_public();
        assert_eq!(
            public.to_string(),
            "C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B"
        );

        let address = Address::from(&public);
        assert_eq!(
            address.to_string(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
        );

        assert_eq!(address.to_public(), public);

        let private: Private = seed.derive(987654321);
        assert_eq!(
            private.to_string(),
            "DDAC3042CAADD9DC480FE3DFB03C21C7144CED51964F33F74B1E79DA727FFAAF"
        );

        let public = private.to_public();
        assert_eq!(
            public.to_string(),
            "93F2893AB61DD7D76B0C9AD081B73946014E382EA87699EC15982A9E468F740A"
        );

        let address = Address::from(&public);
        assert_eq!(
            address.to_string(),
            "nano_36zkj6xde9gqtxois8pii8umkji3brw4xc5pm9p3d83cms5ayx1ciugosdhd"
        );

        let seed =
            Seed::try_from("1BC5FB0ECB41B07AE3272FE2CB037864382167ECE9ECEFB31237EE555627B891")
                .unwrap();
        let address = seed.derive(0).to_public().to_address();
        assert_eq!(
            address.to_string(),
            "nano_1gaki4rjgawxdx7338dsd81f6rebao5qefaonu61jjks6rm1zdrium1f994m"
        );
    }
}
