use crate::blocks::BlockHash;
use crate::encoding::{blake2b, blake2b_callback, deserialize_from_str};
use crate::pow::difficulty::Difficulty;
use crate::{expect_len, hex_formatter, to_hex, Public};
use bytes::Buf;
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug)]
pub enum Subject {
    Hash(BlockHash),
    Public(Public),
}

impl Subject {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Subject::Hash(h) => h.as_bytes(),
            Subject::Public(p) => p.as_bytes(),
        }
    }
}

/// The result of some proof of work (PoW). Can verify and inefficiently generate PoW using the CPU.
#[derive(Clone, PartialEq, Eq)]
pub struct Work([u8; Work::LEN]);

impl Work {
    pub const LEN: usize = 8;

    pub fn zero() -> Self {
        Self([0u8; Self::LEN])
    }

    pub fn random() -> Self {
        let mut s = Self([0u8; Self::LEN]);
        rand::thread_rng().fill_bytes(&mut s.0);
        s
    }

    /// Block and generate forever until we find a solution.
    pub fn generate(subject: &Subject, threshold: &Difficulty) -> anyhow::Result<Work> {
        let mut work_and_subject = [0u8; 40];

        // We can place the subject in the second part of the slice which will not change.
        let subject_slice = &mut work_and_subject[Self::LEN..];

        // This will panic if the source slice is too big. The assertion here is a safety measure
        // in debug builds.
        debug_assert_eq!(subject.as_bytes().len(), subject_slice.len());
        subject.as_bytes().copy_to_slice(subject_slice);

        let mut difficulty: Difficulty = Difficulty::new(0);

        // Fill the first 8 bytes with the random work.
        let work_slice = &mut work_and_subject[0..Self::LEN];
        rand::thread_rng().fill_bytes(work_slice);

        loop {
            // Pick a random byte position and increment.
            // I'm guessing this is slightly faster than using fill_bytes for a new set of numbers.
            // TODO: Bench this guess.
            let idx = (rand::random::<u8>() % (Self::LEN as u8)) as usize;
            let c = work_and_subject[idx];
            work_and_subject[idx] = if c == 0xff { 0 } else { c + 1 };

            blake2b_callback(Self::LEN, &work_and_subject, |b| {
                difficulty = Difficulty::from_le_slice(b).unwrap();
            });
            // TODO: Check if this is > or >=
            if &difficulty > threshold {
                break;
            }
        }

        let work_slice = &work_and_subject[0..Self::LEN];
        let mut work_bytes = Vec::from(work_slice);
        work_bytes.reverse();
        let work = Work::try_from(work_bytes.as_slice()).unwrap();
        return Ok(work);
    }

    pub fn hash(work_and_subject: &[u8]) -> Box<[u8]> {
        blake2b(Self::LEN, work_and_subject)
    }

    pub fn verify(&self, subject: &Subject, threshold: &Difficulty) -> anyhow::Result<bool> {
        let difficulty = self.difficulty(subject)?;
        Ok(&difficulty > threshold)
    }

    pub fn difficulty(&self, subject: &Subject) -> anyhow::Result<Difficulty> {
        let mut work_and_subject = Vec::with_capacity(40);

        // For some reason this is reversed!
        let mut reversed_work = self.0.to_vec();
        reversed_work.reverse();

        work_and_subject.extend_from_slice(&reversed_work);
        work_and_subject.extend_from_slice(subject.as_bytes());
        let hash = Self::hash(&work_and_subject);
        Difficulty::from_le_slice(hash.as_ref())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Work(")?;
        hex_formatter(f, &self.0)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl FromStr for Work {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = hex::decode(s)?;
        let value = vec.as_slice();
        Work::try_from(value)
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

impl Serialize for Work {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_hex(&self.0).as_str())
    }
}

impl<'de> Deserialize<'de> for Work {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_from_str(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Seed;

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
            // This is the same as above except the work is just zeros,
            // causing a totally different difficulty, and not enough work in this case.
            (
                "2387767168f9453db0eca227c79d7e7a31b78cafb58bd9cdee630881c70979ba",
                "0000000000000000",
                "357abcab02726362",
                false,
            ),
        ];

        let threshold = Difficulty::from_str("ffffffc000000000").unwrap();
        for fixture in fixtures {
            let (hash, work, expected_difficulty, is_enough_work) = &fixture;
            let hash = BlockHash::from_str(hash).unwrap();
            let subject = Subject::Hash(hash);
            let work = Work::from_str(work).unwrap();
            let expected_difficulty = Difficulty::from_str(expected_difficulty).unwrap();
            let difficulty = work.difficulty(&subject).unwrap();
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
        // Let's use a low difficulty in debug mode so doesn't take forever.
        let threshold = if cfg!(debug_assertions) {
            Difficulty::from_str("ffff000000000000")
        } else {
            Difficulty::from_str("ffffffc000000000")
        }
        .unwrap();
        dbg!(&threshold);

        let public = Seed::zero().derive(0).to_public().unwrap();
        dbg!(&public);
        let subject = Subject::Public(public);
        let work = Work::generate(&subject, &threshold).unwrap();
        dbg!(&work);
        assert!(work.verify(&subject, &threshold).unwrap());
    }
}
