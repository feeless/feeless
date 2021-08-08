use crate::blocks::BlockHash;
use crate::bytes::Bytes;
use crate::node::{Header, Wire};
use crate::Public;
use anyhow::{anyhow, Error};
use std::convert::TryFrom;

#[derive(Debug)]
struct ExtendedParameters {
    count: u32,
}

#[derive(Debug)]
pub struct BulkPull {
    start: Public,  // TODO: handle when this is a hash
    end: BlockHash, // TODO: anything special to do when this is all 0?
    extended_parameters: Option<ExtendedParameters>,
}

impl BulkPull {
    pub const LEN_BASE: usize = 64;
    pub const LEN_WITH_EXTENSIONS: usize = 72;

    fn has_extended_parameters(header: Option<&Header>) -> anyhow::Result<bool, anyhow::Error> {
        header
            .ok_or(Error::msg("Header required!"))
            .map(|x| x.ext().has_extended_parameters())
    }
}

impl Wire for BulkPull {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut bytes = Bytes::new(data);
        let start: Public = // TODO: handle the case when this is a hash
            Public::try_from(bytes.slice(Public::LEN)?).expect("bulk pull deserializing `start`");
        let end: BlockHash = BlockHash::try_from(bytes.slice(BlockHash::LEN)?)
            .expect("bulk pull deserializing `end`");
        let has_extended_parameters = header
            .ok_or(Error::msg("Header required!"))?
            .ext()
            .has_extended_parameters();
        let extended_parameters: Option<ExtendedParameters> = if has_extended_parameters {
            let zero = bytes.slice(1)?;
            if zero == &[0u8] {
                anyhow!("Bulk Pull message invalid, zero was not 0");
            }
            let mut buffer: [u8; 4] = [0u8; 4];
            let count: &[u8] = bytes.slice(4)?;
            buffer.copy_from_slice(count);
            let count: u32 = u32::from_le_bytes(buffer);
            // TODO: check reserved fields are zero (or maybe not)
            Some(ExtendedParameters { count })
        } else {
            None
        };
        Ok(Self {
            start,
            end,
            extended_parameters,
        })
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        if Self::has_extended_parameters(header)? {
            Ok(Self::LEN_WITH_EXTENSIONS)
        } else {
            Ok(Self::LEN_BASE)
        }
    }
}
