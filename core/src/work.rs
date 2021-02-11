use crate::encoding::blake2b;
use crate::{expect_len, BlockHash, Public};
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq)]
pub struct Difficulty(u64);

impl Difficulty {
    const LEN: usize = 8;
    const HEX_LEN: usize = Self::LEN * 2;

    pub fn new(v: u64) -> Self {
        Self(v)
    }

    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        expect_len(s.len(), Self::HEX_LEN, "Difficulty")?;
        let mut slice = [0u8; Self::LEN];
        hex::decode_to_slice(s, &mut slice)?;
        Self::from_slice(&slice)
    }

    pub fn from_fixed_slice(s: &[u8; Self::LEN]) -> anyhow::Result<Self> {
        Ok(Difficulty(u64::from_le_bytes(*s)))
    }

    pub fn from_slice(s: &[u8]) -> anyhow::Result<Self> {
        let mut b = [0u8; Self::LEN];
        b.copy_from_slice(s);
        Ok(Difficulty(u64::from_le_bytes(b)))
    }

    pub fn is_more_than(&self, threshold: &Difficulty) -> bool {
        println!("{} {}", hex::encode(&self.0.to_le_bytes()), &self.0);
        println!(
            "{} {} (min)",
            hex::encode(&threshold.0.to_le_bytes()),
            &threshold.0
        );
        self.0 > threshold.0
    }
}

#[derive(Debug)]
pub enum Subject {
    Hash(BlockHash),
    Public(Public),
}

impl Subject {
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            Subject::Hash(h) => h.as_bytes(),
            Subject::Public(p) => p.as_bytes(),
        }
    }
}

#[derive(Debug)]
pub struct Work([u8; Work::LEN]);

impl Work {
    pub const LEN: usize = 8;

    pub fn zero() -> Self {
        Self([0u8; Self::LEN])
    }

    pub fn from_hex(s: &str) -> anyhow::Result<Self> {
        let mut value = hex::decode(s)?;
        value.reverse();
        let value = value.as_slice();
        Work::try_from(value)
    }

    pub fn generate(subject: &Subject, difficulty: u32) -> Work {
        todo!()
    }

    pub fn hash(work_and_subject: &[u8]) -> Box<[u8]> {
        blake2b(Self::LEN, work_and_subject)
    }

    pub fn verify(&self, subject: &Subject, threshold: &Difficulty) -> anyhow::Result<bool> {
        let difficulty = self.get_difficulty(subject)?;
        Ok(difficulty.is_more_than(threshold))
    }

    // This is very probably not performant, but I'm just here to make it work first.
    pub fn get_difficulty(&self, subject: &Subject) -> anyhow::Result<Difficulty> {
        let mut work_and_subject = Vec::new();
        work_and_subject.extend_from_slice(&self.0);
        work_and_subject.extend_from_slice(subject.to_bytes());
        let mut hash = Self::hash(&work_and_subject);
        hash.reverse();
        Difficulty::from_slice(hash.as_ref())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Work {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Work")?;

        let mut s = Work::zero();
        s.0.copy_from_slice(value);
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify() {
        // Each hash is incremented by one.
        let fixtures = vec![
            (
                "2387767168f9453db0eca227c79d7e7a31b78cafb58bd9cdee630881c70979b8",
                "c3f097857cc7106b",
                "fffffff867b3146b",
                true,
            ),
            (
                "2387767168f9453db0eca227c79d7e7a31b78cafb58bd9cdee630881c70979b9",
                "ec4f0960a70fdcbe",
                "fffffffde26451db",
                true,
            ),
            (
                "2387767168f9453db0eca227c79d7e7a31b78cafb58bd9cdee630881c70979ba",
                "b58e13f297179bc2",
                "fffffffb6fc1b4a6",
                true,
            ),
            // This is the same as above except the work has its first byte modified,
            // causing a totally different difficulty, and not enough work.
            (
                "2387767168f9453db0eca227c79d7e7a31b78cafb58bd9cdee630881c70979ba",
                "c58e13f297179bc2",
                "3b24d56cc1f19103",
                false,
            ),
        ];

        let threshold = Difficulty::from_hex("ffffffc000000000").unwrap();
        for fixture in fixtures {
            let (hash, work, expected_difficulty, is_enough_work) = &fixture;
            let hash = BlockHash::from_hex(hash).unwrap();
            let subject = Subject::Hash(hash);
            let work = Work::from_hex(work).unwrap();
            let expected_difficulty = Difficulty::from_hex(expected_difficulty).unwrap();

            let difficulty = work.get_difficulty(&subject).unwrap();

            assert_eq!(difficulty, expected_difficulty, "{:?}", &fixture);

            assert_eq!(
                work.verify(&subject, &threshold).unwrap(),
                *is_enough_work,
                "{:?}",
                &fixture
            );
        }
    }

    #[test]
    fn generate_work() {
        // let subject = Subject::
        // let work = Work::generate();
        // work.verify()
    }
}
