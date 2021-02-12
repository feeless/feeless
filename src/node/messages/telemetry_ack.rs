use crate::node::header::Header;
use crate::node::wire::Wire;
use crate::{to_hex, BlockHash, Private, Public, Seed, Signature};
use tracing::warn;

#[derive(Debug)]
pub struct TelemetryAck([u8; Self::LEN]);

// {
//     signature: Signature,
//     node_id: Public,
//     block_count: u64,
//     cemented_count: u64,
//     unchecked_count: u64,
//     account_count: u64,
//     bandwidth_cap: u64,
//     uptime: u64,
//     peer_count: u32,
//     protocol_version: u8,
//     genesis_block: BlockHash,
//     major_version: u8,
//     minor_version: u8,
//     patch_version: u8,
//     prerelease_version: u8,
//     maker: u8,
//     timestamp: [u8; 8],
//     active_difficulty: [u8; 8],
// }

impl TelemetryAck {
    pub const LEN: usize = 200;
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
        Ok(Self([0u8; Self::LEN]))
    }

    fn len(header: Option<&Header>) -> Result<usize, anyhow::Error>
    where
        Self: Sized,
    {
        warn!("telemetry ack len");
        Ok(TelemetryAck::LEN)
    }
}
