use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{to_hex, BlockHash, Private, Public, Seed, Signature};
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
    pub const LEN: usize = 204;
}

impl Default for TelemetryAck {
    // TODO: Temporary only...
    fn default() -> Self {
        Self {
            signature: Signature::zero(),
            node_id: Seed::zero().derive(0).to_public(),
            block_count: 0,
            cemented_count: 0,
            unchecked_count: 0,
            account_count: 0,
            bandwidth_cap: 0,
            uptime: 0,
            peer_count: 0,
            protocol_version: 0,
            genesis_block: BlockHash::from_hex(
                "7837C80964CAD551DEABE162C7FC4BB58688A0C6EB6D9907C0D2A7C74A33C7EB",
            )
            .unwrap(),
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
            prerelease_version: 0,
            maker: 0,
            timestamp: [0u8; 8],
            active_difficulty: [0u8; 8],
        }
    }
}

impl Wire for TelemetryAck {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        warn!("TODO deserialize telemetry ack");
        println!("{}", to_hex(data));
        Ok(Self::default())
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error>
    where
        Self: Sized,
    {
        warn!("telemetry ack len");
        Ok(TelemetryAck::LEN)
    }
}
