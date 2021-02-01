use crate::key::Private;
use std::convert::TryFrom;
use hex;
use blake2::{Blake2b, Digest, VarBlake2b};
use bytes::{BytesMut, BufMut};
use blake2::digest::{Update, VariableOutput};

pub struct Seed(pub [u8; 32]);

impl Seed {
    fn zero() -> Self {
        Self([0; 32])
    }

    /// Derive a private key from the seed with an index.
    ///
    /// https://docs.nano.org/integration-guides/the-basics/#seed
    pub fn derive(&self, index: u32) -> Private {
        let mut buf = BytesMut::with_capacity(32 + 4);  // seed + index
        buf.put(self.0.as_ref());
        buf.put_u32(index);

        // Expect this to work because the output size >0.
        let mut blake = VarBlake2b::new(32).expect("output size was zero");
        blake.update(&buf);
        let result = blake.finalize_boxed();

        // Expect this to work all the time because it's coming from known correct types.
        Private::try_from(result.as_ref()).expect("conversion from seed")
    }
}

impl TryFrom<&str> for Seed {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut seed = Seed::zero();
        hex::decode_to_slice(value, &mut seed.0)?;
        Ok(seed)
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::fmt_hex(f, &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use crate::Public;
    use crate::Address;

    #[test]
    fn conversions() {
        let seed = Seed::try_from("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let private = seed.derive(0);
        assert_eq!(private.to_string(), "9F0E444C69F77A49BD0BE89DB92C38FE713E0963165CCA12FAF5712D7657120F");

        let public = Public::from(&private);
        assert_eq!(public.to_string(), "C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B");

        let address = Address::from(&public);
        assert_eq!(public.to_string(), "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7");

        let private = seed.derive(987654321);
        assert_eq!(private.to_string(), "DDAC3042CAADD9DC480FE3DFB03C21C7144CED51964F33F74B1E79DA727FFAAF");

        let public = Public::from(&private);
        assert_eq!(public.to_string(), "93F2893AB61DD7D76B0C9AD081B73946014E382EA87699EC15982A9E468F740A");

        let address = Address::from(&public);
        assert_eq!(public.to_string(), "nano_36zkj6xde9gqtxois8pii8umkji3brw4xc5pm9p3d83cms5ayx1ciugosdhd");
    }
}