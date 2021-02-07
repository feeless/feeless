use anyhow::anyhow;

pub struct Bytes<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn slice(&mut self, size: usize) -> anyhow::Result<&[u8]> {
        if self.offset + size >= self.bytes.len() {
            return Err(anyhow!(
                "Slice extended past end. offset: {} size: {} len: {}",
                self.offset,
                size,
                self.bytes.len()
            ));
        }

        let bytes = &self.bytes[self.offset..self.offset + size];
        self.offset += size;
        Ok(bytes)
    }

    pub fn u8(&mut self) -> anyhow::Result<u8> {
        let b = self.bytes[self.offset];
        self.offset += 1;
        Ok(b)
    }
}
