use crate::encoding::blake2b;
use crate::hexify;
use crate::Private;
use bytes::{BufMut, BytesMut};
use rand::RngCore;
use std::convert::TryFrom;
use std::str::FromStr;

/// 256 bit seed used to derive multiple addresses.
///
/// See https://docs.nano.org/integration-guides/the-basics/#seed for details.
#[derive(Clone, PartialEq)]
pub struct Seed(pub [u8; Seed::LEN]);

hexify!(Seed, "seed");

impl Seed {
    const LEN: usize = 32;

    pub fn zero() -> Self {
        Self([0; Self::LEN])
    }

    /// Generate a secure random seed.
    pub fn random() -> Self {
        let mut seed = Seed::zero();
        rand::thread_rng().fill_bytes(&mut seed.0);
        seed
    }

    /// Derive a private key from the seed with an index.
    ///
    /// https://docs.nano.org/integration-guides/the-basics/#seed
    pub fn derive(&self, index: u32) -> Private {
        let mut buf = BytesMut::with_capacity(Self::LEN + 4); // seed + index
        buf.put(self.0.as_ref());
        buf.put_u32(index);

        let result = blake2b(Self::LEN, &buf);

        // Expect this to work all the time because it's coming from known correct types.
        Private::try_from(result.as_ref()).expect("conversion from seed")
    }
}
