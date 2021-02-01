use blake2::digest::{Update, VariableOutput};
use blake2::{Digest, VarBlake2b};

pub fn fmt_hex(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02X}", byte)?;
    }
    Ok(())
}

pub fn blake2b(size: usize, data: &[u8]) -> Box<[u8]> {
    let mut blake = VarBlake2b::new(size).expect("output size was zero");
    blake.update(&data);
    blake.finalize_boxed()
}
