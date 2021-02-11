use crate::encoding::blake2b;
use crate::{expect_len, Private};
use bip39::{Language, Mnemonic, MnemonicType};
use bytes::{BufMut, BytesMut};
use rand::RngCore;
use std::convert::TryFrom;

pub struct Seed(pub [u8; Seed::LEN]);

impl Seed {
    const LEN: usize = 32;

    pub(crate) fn zero() -> Self {
        Self([0; Self::LEN])
    }

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

impl TryFrom<&str> for Seed {
    type Error = anyhow::Error;

    /// Expecting a hex string.
    /// TODO: Move into Seed::from_hex()
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        expect_len(value.len(), Seed::LEN * 2, "Seed")?;
        let mut seed = Seed::zero();
        hex::decode_to_slice(value, &mut seed.0)?;
        Ok(seed)
    }
}

impl TryFrom<&[u8]> for Seed {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Seed::LEN, "Seed")?;
        let mut seed = Seed::zero();
        seed.0.copy_from_slice(value);
        Ok(seed)
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::encoding::hex_formatter(f, &self.0)
    }
}
