use crate::{Phrase, Private, Seed};
use std::path::PathBuf;

pub enum Source {
    Phrase(Phrase),
    Seed(Seed),
    Private(Private),
}

pub struct Wallet {
    source: Source,
    path: PathBuf,
}
