use crate::blocks::{Block, BlockHash, BlockType};
use crate::bytes::Bytes;
use crate::encoding::blake2b;
use crate::node::header::Header;
use crate::node::timestamp::Timestamp;
use crate::node::wire::Wire;
use crate::{Public, Signature};
use anyhow::Context;
use std::convert::TryFrom;
use tracing::trace;

/// This is a vote on the network by a representative for one or more block hashes.
#[derive(Debug)]
pub struct ConfirmAck {
    pub account: Public,
    pub signature: Signature,

    pub timestamp: Timestamp,
    pub confirm: Confirm,
}

#[derive(Debug)]
pub enum Confirm {
    VoteByHash(Vec<BlockHash>),

    // TODO: This looks like it isn't used on the live network.
    Block(Block),
}

impl ConfirmAck {
    const VOTE_COMMON_LEN: usize = Public::LEN + Signature::LEN + Timestamp::LEN;

    pub fn new(
        account: Public,
        signature: Signature,
        timestamp: Timestamp,
        confirm: Confirm,
    ) -> Self {
        Self {
            account,
            signature,
            timestamp,
            confirm,
        }
    }

    pub fn verify_signature(&self) -> anyhow::Result<()> {
        self.account
            .verify(&self.inner_hash(), &self.signature)
            .context("Verify signature on ConfirmAck")
    }

    // nano::block_hash nano::vote::hash () const
    pub fn inner_hash(&self) -> Vec<u8> {
        let mut v = Vec::new();

        // TODO: Only add this prefix if there's data. See nano::vote::hash()
        v.extend_from_slice("vote ".as_bytes());

        if let Confirm::VoteByHash(hashes) = &self.confirm {
            for hash in hashes {
                v.extend_from_slice(hash.as_bytes())
            }
            v.extend_from_slice(&self.timestamp.to_bytes())
        } else {
            todo!("handle block hash");
        }

        blake2b(BlockHash::LEN, &v).to_vec()
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
        debug_assert!(header.is_some());
        let header = header.unwrap();

        let mut data = Bytes::new(data);
        let account = Public::try_from(data.slice(Public::LEN)?)?;
        let signature = Signature::try_from(data.slice(Signature::LEN)?)?;
        // `to_vec` here to stop a borrow problem
        // Looks like this is "sequence" on the live network, but will change to "timestamp".
        let timestamp = Timestamp::try_from(data.slice(Timestamp::LEN)?)?;
        let confirm = if header.ext().block_type()? == BlockType::NotABlock {
            let mut block_hashes = vec![];
            for _ in 0..header.ext().item_count() {
                block_hashes.push(BlockHash::try_from(data.slice(BlockHash::LEN)?)?);
            }
            Confirm::VoteByHash(block_hashes)
        } else {
            // let block = Block;
            dbg!("block!!!!!!!", header.ext().block_type().unwrap());
            todo!()
        };

        Ok(Self::new(account, signature, timestamp, confirm))
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize> {
        debug_assert!(header.is_some());
        let header = header.unwrap();

        if header.ext().block_type()? == BlockType::NotABlock {
            Ok(Self::VOTE_COMMON_LEN + header.ext().item_count() * BlockHash::LEN)
        } else {
            todo!("got a block in confirm ack {:#?}", header);
            // Ok(Self::VOTE_COMMON_LEN + FullBlock::LEN)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn verify_sig() {
        let account =
            Public::from_str("2994D330022A052DF83E10FCE1B3E140496CDCD7E0C0F2FF6DE2670291B88011")
                .unwrap();
        let signature = Signature::from_str("721C6CAFD61C2D7ED27643C556F77AE900308BD5AAF458E74310E42773BB45494A138EE0291B6868C360EB983AB5CE8FF2EFF6A66044CBA2B128047ACDBD4402").unwrap();
        let hash1 =
            BlockHash::from_str("C3A3FE56D584CB997199E3B09EC454F62DED3B7EF875D9D7E8E5011AC34C77A5")
                .unwrap();
        let hash2 =
            BlockHash::from_str("139E1064D7CCC26495EFB4030015C02CE78556EBE3547192843B0E71C91599FC")
                .unwrap();
        let timestamp = Timestamp::from_u64(2019626603);
        let confirm_ack = ConfirmAck::new(
            account,
            signature,
            timestamp,
            Confirm::VoteByHash(vec![hash1, hash2]),
        );
        assert!(confirm_ack.verify_signature().is_ok());
    }
}
