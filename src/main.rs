#![forbid(unsafe_code)]

mod raw;
mod key;
mod seed;
mod address;

use address::Address;
use key::{Public, Private};
use blake2::{Blake2b, Digest, VarBlake2b};
use blake2::digest::{Update, VariableOutput};

fn main() {
    println!("Hello, world!");
}

fn fmt_hex(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02X}", byte)?;
    }
    Ok(())
}

fn blake2b(size: usize, data: &[u8]) -> Box<[u8]> {
    let mut blake = VarBlake2b::new(size).expect("output size was zero");
    blake.update(&data);
    blake.finalize_boxed()
}