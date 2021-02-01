mod raw;
mod key;
mod seed;

fn main() {
    println!("Hello, world!");
}

fn fmt_hex(f: &mut std::fmt::Formatter<'_>, bytes: &[u8]) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{:02X}", byte)?;
    }
    Ok(())
}
