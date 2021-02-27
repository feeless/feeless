use crate::node::header::Header;
use crate::node::wire::Wire;
use anyhow::anyhow;
use std::convert::TryFrom;
use std::fmt::Debug;

#[derive(Debug)]
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
        // TODO: make this safer--maybe use replace usize with u32 so it's always smaller than i64.
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
                "Slice extended past end. Offset: {} Requested size: {} Bytes len: {} Remain: {}",
                self.offset,
                size,
                self.bytes.len(),
                self.remain(),
            ))
        } else {
            Ok(())
        }
    }

    // pub fn wire<T>(&mut self, header: Option<&Header>) -> anyhow::Result<T>
    // where
    //     T: Wire + TryFrom<&'a [u8]> + ToOwned + ToOwned<Owned = T>,
    //     <T as TryFrom<&'a [u8]>>::Error: Debug,
    // {
    //     let len = T::len(header)?;
    //     let slice = self.slice(len)?.to_owned();
    //     let t = T::try_from(&slice).unwrap(); // .map_err(|e| anyhow!("Try from slice: {:?} {:?}", e, &self))?;
    //     Ok(t.clone())
    // }
}
