use crate::blocks::{Block, BlockType};
use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::timestamp::IncrementalTimestamp;
use crate::node::wire::Wire;
use crate::{BlockHash, Public, Signature};
use std::convert::TryFrom;
use tracing::trace;

#[derive(Debug)]
pub struct ConfirmAck {
    pub account: Public,
    pub signature: Signature,

    pub timestamp: IncrementalTimestamp,
    pub confirm: Confirm,
}

#[derive(Debug)]
pub enum Confirm {
    VoteByHash(Vec<BlockHash>),
    Block(Block), // TODO: this variant is 704 bytes, use Box<>?
}

impl ConfirmAck {
    const VOTE_COMMON_LEN: usize = Public::LEN + Signature::LEN + IncrementalTimestamp::LEN;

    pub fn new(
        account: Public,
        signature: Signature,
        timestamp: IncrementalTimestamp,
        confirm: Confirm,
    ) -> Self {
        Self {
            account,
            signature,
            timestamp,
            confirm,
        }
    }

    pub fn verify(&self) -> anyhow::Result<()> {
        // self.account.verify(self.confirm)
        todo!()
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice("vote ".as_bytes());

        if let Confirm::VoteByHash(hashes) = &self.confirm {
            for hash in hashes {
                v.extend_from_slice(hash.as_bytes())
            }
            // TODO
            // v.extend_from_slice(timestamp.as_bytes())
        } else {
            todo!("handle block hash");
        }

        todo!()
    }
}

impl Wire for ConfirmAck {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        trace!("deserialize confirm ack");
        debug_assert!(header.is_some());
        let header = header.unwrap();

        let mut data = Bytes::new(data);
        let account = Public::try_from(data.slice(Public::LEN)?)?;
        let signature = Signature::try_from(data.slice(Signature::LEN)?)?;
        // to_vec here to stop a borrow problem
        let timestamp = IncrementalTimestamp::try_from(data.slice(IncrementalTimestamp::LEN)?)?;
        let confirm = if header.ext().block_type()? == BlockType::NotABlock {
            let mut block_hashes = vec![];
            for _ in 0..header.ext().item_count() {
                block_hashes.push(BlockHash::try_from(data.slice(BlockHash::LEN)?)?);
            }
            Confirm::VoteByHash(block_hashes)
        } else {
            // let block = Block;
            trace!("block");
            dbg!("block!!!!!!!", header.ext().block_type().unwrap());
            // dbg!("{:X}", data.slice(FullBlock::LEN)?);
            todo!()
        };

        Ok(Self::new(account, signature, timestamp, confirm))
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize> {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type()? == BlockType::NotABlock {
            trace!("not a block");
            Ok(Self::VOTE_COMMON_LEN + header.ext().item_count() * BlockHash::LEN)
        } else {
            trace!("a block");
            dbg!(header);
            todo!("got a block in confirm ack");
            // Ok(Self::VOTE_COMMON_LEN + FullBlock::LEN)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::FromHex;

    #[test]
    fn verify_sig() {
        let account =
            Public::from_hex("96B8D493E24886F9B52919C40D169B1B914CEAD7D064AFBA916264C87A305A56")
                .unwrap();
        let signature = Signature::from_hex("5A8FFB1F0F8CD7900A9703D2984963CA560E34ED414149AC2EC8666E55D28BEBE79F59F7949345DE5A7DD7B9FA4408F57CCEC44458731AC52927C6525878DA05").unwrap();
        let block_hash =
            BlockHash::from_hex("3332DE6136266EDB713439599E7F5F0ADAC2B08CEDAF1104F542854D33A81833")
                .unwrap();
        let confirm_ack = ConfirmAck::new(
            account,
            signature,
            IncrementalTimestamp::new(),
            Confirm::VoteByHash(vec![block_hash]),
        );
        assert!(confirm_ack.verify().is_ok());
    }
}
