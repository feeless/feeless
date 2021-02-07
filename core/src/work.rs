#[derive(Debug)]
pub struct Work([u8; Work::LEN]);

impl Work {
    pub const LEN: usize = 8;

    pub fn zero() -> Self {
        Self([0u8; 8])
    }
}
