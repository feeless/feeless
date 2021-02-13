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

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn remain(&self) -> usize {
        self.len() - self.offset()
    }

    pub fn seek(&mut self, amount: i64) -> anyhow::Result<()> {
        self.bounds_check(amount)?;
        self.offset = (self.offset as i64 + amount) as usize;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn slice(&mut self, size: usize) -> anyhow::Result<&[u8]> {
        self.bounds_check(size as i64)?;
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

    fn bounds_check(&mut self, size: i64) -> anyhow::Result<()> {
        if (self.offset as i64 + size) as usize > self.bytes.len() {
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
