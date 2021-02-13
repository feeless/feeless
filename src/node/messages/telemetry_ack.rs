use crate::bytes::Bytes;
use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{to_hex, BlockHash, Private, Public, Seed, Signature};
use anyhow::Context;
use std::convert::TryFrom;
use tracing::warn;

#[derive(Debug)]
pub struct TelemetryAck {
    signature: Signature,
    node_id: Public,
    block_count: u64,
    cemented_count: u64,
    unchecked_count: u64,
    account_count: u64,
    bandwidth_cap: u64,
    uptime: u64,
    peer_count: u32,
    protocol_version: u8,
    genesis_block: BlockHash,
    major_version: u8,
    minor_version: u8,
    patch_version: u8,
    prerelease_version: u8,
    maker: u8,
    timestamp: [u8; 8],
    active_difficulty: [u8; 8],
}

impl TelemetryAck {
    pub const LEN: usize = 202;
}

impl Wire for TelemetryAck {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut bytes = Bytes::new(data);

        let mut s = Self {
            signature: Signature::try_from(bytes.slice(Signature::LEN)?)
                .context("Telemetry ack decoding signature")?,
            node_id: Public::try_from(bytes.slice(Public::LEN)?)
                .context("Telemetry ack decoding node_id")?,
            block_count: 0,
            cemented_count: 0,
            unchecked_count: 0,
            account_count: 0,
            bandwidth_cap: 0,
            uptime: 0,
            peer_count: 0,
            protocol_version: 0,
            genesis_block: BlockHash::zero(),
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
            prerelease_version: 0,
            maker: 0,
            timestamp: [0u8; 8],
            active_difficulty: [0u8; 8],
        };

        let mut s32 = [0u8; 4];
        let mut s64 = [0u8; 8];

        s64.copy_from_slice(bytes.slice(8)?);
        s.block_count = u64::from_be_bytes(s64);
        s64.copy_from_slice(bytes.slice(8)?);
        s.cemented_count = u64::from_be_bytes(s64);
        s64.copy_from_slice(bytes.slice(8)?);
        s.unchecked_count = u64::from_be_bytes(s64);
        s64.copy_from_slice(bytes.slice(8)?);
        s.account_count = u64::from_be_bytes(s64);
        s64.copy_from_slice(bytes.slice(8)?);
        s.bandwidth_cap = u64::from_be_bytes(s64);
        s64.copy_from_slice(bytes.slice(8)?);
        s.uptime = u64::from_be_bytes(s64);
        s32.copy_from_slice(bytes.slice(4)?);
        s.peer_count = u32::from_be_bytes(s32);
        s.protocol_version = bytes.u8()?;
        s.genesis_block = BlockHash::try_from(bytes.slice(BlockHash::LEN)?)
            .context("Telemetry ack decoding genesis block")?;

        s.major_version = bytes.u8()?;
        s.minor_version = bytes.u8()?;
        s.patch_version = bytes.u8()?;
        s.prerelease_version = bytes.u8()?;
        s.maker = bytes.u8()?;

        warn!("TODO: telemetry ack timestamp");
        warn!("TODO: telemetry ack active_difficulty");

        Ok(s)
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(TelemetryAck::LEN)
    }
}
