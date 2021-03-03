use std::convert::{TryFrom, TryInto};
use std::result::Result;

use anyhow::{anyhow, Context};
use bitvec::prelude::*;

use crate::blocks::BlockType;
use crate::expect_len;
use crate::network::Network;
use crate::node::wire::Wire;

// TODO: Have header internally only contain [u8; 8] and use accessors, so that the header doesn't
//       have to be encoded/decoded when sending/receiving.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Header {
    /// Always "R" 0x82, probably for RaiBlocks!
    magic_number: MagicNumber,

    /// Network: live (C 0x43), beta (B 0x42), test (A 0x41).
    /// https://github.com/nanocurrency/nano-node/blob/8c650ee8f537c3ded9a4a518f5f7df56c6a67904/nano/secure/common.cpp#L89
    network: Network,

    /// Protocol version
    /// https://github.com/nanocurrency/nano-node/blob/8c650ee8f537c3ded9a4a518f5f7df56c6a67904/nano/secure/common.hpp#L350
    version_max: Version,
    version_using: Version,
    version_min: Version,

    /// Type of data in the payload.
    /// https://github.com/nanocurrency/nano-node/blob/8c650ee8f537c3ded9a4a518f5f7df56c6a67904/nano/node/common.hpp#L162
    message_type: MessageType,

    /// Extra data in bits.
    ext: Extensions,
}

impl Header {
    pub fn validate(&self, network: &Network) -> anyhow::Result<()> {
        if &self.network != network {
            return Err(anyhow!(
                "network mismatch: They're on {:?}. We're on {:?}",
                self.network,
                network,
            ));
        }

        // TODO: Check versions.

        Ok(())
    }

    pub fn to_short_string(&self) -> String {
        format!("{:?} {:?}", self.message_type, self.ext)
    }
}

impl Header {
    pub const LEN: usize = 8;

    // Header offsets.
    const MAGIC_NUMBER: usize = 0;
    const NETWORK: usize = 1;
    const VERSION_MAX: usize = 2;
    const VERSION_USING: usize = 3;
    const VERSION_MIN: usize = 4;
    const MESSAGE_TYPE: usize = 5;
    const EXTENSIONS: usize = 6;

    pub fn new(network: Network, message_type: MessageType, ext: Extensions) -> Self {
        Self {
            magic_number: MagicNumber::new(),
            network,
            version_max: Version::V18,
            version_using: Version::V18,
            version_min: Version::V18,
            message_type,
            ext,
        }
    }

    pub fn reset(&mut self, message_type: MessageType, ext: Extensions) -> &mut Self {
        self.message_type = message_type;
        self.ext = ext;
        self
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    pub fn ext(&self) -> Extensions {
        self.ext
    }
}

impl Wire for Header {
    fn serialize(&self) -> Vec<u8> {
        vec![
            self.magic_number.0,
            self.network as u8,
            self.version_max as u8,
            self.version_using as u8,
            self.version_min as u8,
            self.message_type as u8,
            self.ext.0[0],
            self.ext.0[1],
        ]
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self> {
        debug_assert!(header.is_none());

        let context = || format!("Deserializing header");

        expect_len(data.len(), Header::LEN, "Header")?;
        MagicNumber::try_from(data[Self::MAGIC_NUMBER]).with_context(context)?;

        let network = Network::try_from(data[Self::NETWORK]).with_context(context)?;
        let message_type = MessageType::try_from(data[Self::MESSAGE_TYPE])?;
        let ext =
            Extensions::try_from(&data[Self::EXTENSIONS..Self::EXTENSIONS + Extensions::LEN])?;

        Ok(Header::new(network, message_type, ext))
    }

    fn len(_: Option<&Header>) -> anyhow::Result<usize> {
        Ok(Header::LEN)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct MagicNumber(u8);

impl MagicNumber {
    const MAGIC: u8 = 0x52;

    pub fn new() -> Self {
        Self(Self::MAGIC)
    }
}

impl std::fmt::Debug for MagicNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0)?;
        Ok(())
    }
}

impl TryFrom<u8> for MagicNumber {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if v != Self::MAGIC {
            return Err(anyhow!("Invalid magic number: {}", v));
        }
        Ok(Self::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Version {
    V18 = 18,
    V19 = 19,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    Keepalive = 2,
    Publish = 3,
    ConfirmReq = 4,
    ConfirmAck = 5,
    BulkPull = 6,
    BulkPush = 7,
    FrontierReq = 8,

    /// A NodeIdHandshake shares a cookie to other peers, which is then responded with
    /// the node giving out their public key and a signed message of the cookie.
    Handshake = 10,

    BulkPullAccount = 11,
    TelemetryReq = 12,
    TelemetryAck = 13,
}

impl TryFrom<u8> for MessageType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use MessageType::*;
        Ok(match value {
            2 => Keepalive,
            3 => Publish,
            4 => ConfirmReq,
            5 => ConfirmAck,
            6 => BulkPull,
            7 => BulkPush,
            8 => FrontierReq,
            10 => Handshake,
            11 => BulkPullAccount,
            12 => TelemetryReq,
            13 => TelemetryAck,
            v => return Err(anyhow!("Unknown message type: {}", v)),
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Extensions([u8; 2]);

impl Extensions {
    const LEN: usize = 2;

    // Bit offsets and lengths
    const QUERY: usize = 0;
    const RESPONSE: usize = 1;
    const ITEM_COUNT: usize = 12;
    const ITEM_COUNT_BITS: usize = 4;
    const BLOCK_TYPE: usize = 8;
    const BLOCK_TYPE_BITS: usize = 4;

    pub fn new() -> Self {
        Self([0, 0])
    }

    pub fn query(&mut self) -> &mut Self {
        self.mut_bits().set(Self::QUERY, true);
        self
    }

    pub fn is_query(&self) -> bool {
        self.bits()[Self::QUERY]
    }

    pub fn response(&mut self) -> &mut Self {
        self.mut_bits().set(Self::RESPONSE, true);
        self
    }

    pub fn is_response(&self) -> bool {
        self.bits()[Self::RESPONSE]
    }

    pub fn item_count(&self) -> usize {
        self.bits()[Self::ITEM_COUNT..Self::ITEM_COUNT + Self::ITEM_COUNT_BITS].load_be()
    }

    pub fn block_type(&self) -> anyhow::Result<BlockType> {
        self.bits()[Self::BLOCK_TYPE..Self::BLOCK_TYPE + Self::BLOCK_TYPE_BITS]
            .load_be::<u8>()
            .try_into()
    }

    fn bits(&self) -> &BitSlice<Lsb0, u8> {
        self.0.view_bits()
    }

    fn mut_bits(&mut self) -> &mut BitSlice<Lsb0, u8> {
        self.0.view_bits_mut()
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = vec![];
        if self.is_query() {
            s.push("Query")
        }
        if self.is_response() {
            s.push("Response")
        }
        write!(f, "[{}]", s.join(", "))?;

        Ok(())
    }
}

impl TryFrom<&[u8]> for Extensions {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Extensions")?;

        let mut s = Self::new();
        s.0[0] = value[0];
        s.0[1] = value[1];
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::node::state::MemoryState;
    use crate::node::state::State;

    use super::*;

    #[test]
    fn serialize() {
        let network = Network::Live;
        let state = MemoryState::new(network);

        let ext = *Extensions::new().query().response();
        let h1 = Header::new(network, MessageType::Keepalive, ext);
        let s = h1.serialize();
        assert_eq!(s.len(), Header::LEN);
        assert_eq!(s, vec![0x52, 0x43, 18, 18, 18, 2, 3, 0]);

        let h2 = Header::deserialize(None, &s).unwrap();
        assert_eq!(h1, h2);
    }

    fn assert_contains_err<T: Debug>(result: anyhow::Result<T>, s: &str) {
        for entry in result.as_ref().err().as_ref().unwrap().chain() {
            if entry.to_string().contains(s) {
                return;
            }
        }
        assert!(
            false,
            format!(
                "Got error:\n{:?}\n\nExpecting: {}",
                &result.err().unwrap(),
                s
            )
        );
    }

    #[test]
    fn bad_length() {
        let err = "Header is the wrong length";
        let s = vec![];
        assert_contains_err(Header::deserialize(None, &s), err);
        let s = vec![0xFF, 0x43, 18, 18, 18, 2, 3, 0, 0xFF];
        assert_contains_err(Header::deserialize(None, &s), err);
    }

    #[test]
    fn bad_magic() {
        let s = vec![0xFF, 0x43, 18, 18, 18, 2, 3, 0];
        assert_contains_err(Header::deserialize(None, &s), "magic number");
    }

    #[test]
    fn bad_network() {
        let s = vec![0x52, 0x43, 18, 18, 18, 2, 3, 0];
        let header = Header::deserialize(None, &s).unwrap();
        let result = header.validate(&Network::Test);
        assert_contains_err(result, "network mismatch");
    }

    #[test]
    fn bad_message_type() {
        let s = vec![0x52, 0x43, 18, 18, 18, 100, 3, 0];
        assert_contains_err(Header::deserialize(None, &s), "message type");
    }

    #[test]
    fn item_count() {
        let fixtures: &[(u8, u8, u8)] = &[
            (0x00, 0x00, 0),
            (0xff, 0xff, 15),
            (0x00, 0xff, 15),
            (0xff, 0xa0, 10),
            (0xff, 0x50, 5),
            (0xff, 0x10, 1),
        ];
        for (b1, b2, expected) in fixtures {
            dbg!(b1, b2, expected);
            let ext = Extensions::try_from([*b1, *b2].as_ref()).unwrap();
            assert_eq!(ext.item_count() as u8, *expected);
        }
    }
}
