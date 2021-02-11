use anyhow::anyhow;

pub struct Bytes<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn eof(&self) -> bool {
        self.offset >= self.bytes.len()
    }

    pub fn slice(&mut self, size: usize) -> anyhow::Result<&[u8]> {
        self.bounds_check(size)?;
        let bytes = &self.bytes[self.offset..self.offset + size];
        self.offset += size;
        Ok(bytes)
    }

    pub fn u8(&mut self) -> anyhow::Result<u8> {
        self.bounds_check(1)?;
        let b = self.bytes[self.offset];
        self.offset += 1;
        Ok(b)
    }

    fn bounds_check(&mut self, size: usize) -> anyhow::Result<()> {
        if self.offset + size > self.bytes.len() {
            Err(anyhow!(
                "slice extended past end. offset: {} size: {} len: {}",
                self.offset,
                size,
                self.bytes.len()
            ))
        } else {
            Ok(())
        }
    }
}
