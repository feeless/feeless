use std::convert::TryFrom;

use bytes::{BufMut, BytesMut};

use crate::encoding::blake2b;
use crate::Private;

const SEED_BYTES: usize = 32;

pub struct Seed(pub [u8; SEED_BYTES]);

impl Seed {
    fn zero() -> Self {
        Self([0; SEED_BYTES])
    }

    /// Derive a private key from the seed with an index.
    ///
    /// https://docs.nano.org/integration-guides/the-basics/#seed
    pub fn derive(&self, index: u32) -> Private {
        let mut buf = BytesMut::with_capacity(SEED_BYTES + 4); // seed + index
        buf.put(self.0.as_ref());
        buf.put_u32(index);

        let result = blake2b(SEED_BYTES, &buf);

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
        crate::encoding::fmt_hex(f, &self.0)
    }
}
