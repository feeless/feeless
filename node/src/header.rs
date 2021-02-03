use crate::state::State;
use crate::wire::Wire;
use anyhow::anyhow;
use bitvec::prelude::*;
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::result::Result;

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
    flags: Flags,
}

impl Header {
    pub const LENGTH: usize = 8;

    // Header offsets.
    const MAGIC_NUMBER: usize = 0;
    const NETWORK: usize = 1;
    const VERSION_MAX: usize = 2;
    const VERSION_USING: usize = 3;
    const VERSION_MIN: usize = 4;
    const MESSAGE_TYPE: usize = 5;
    const FLAGS: usize = 6;

    pub fn new(network: Network, message_type: MessageType, flags: Flags) -> Self {
        Self {
            magic_number: MagicNumber::new(),
            network,
            version_max: Version::Current,
            version_using: Version::Current,
            version_min: Version::Current,
            message_type,
            flags,
        }
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    pub fn flags(&self) -> Flags {
        self.flags
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
            self.flags.0[0],
            self.flags.0[1],
        ]
    }

    fn deserialize(state: &State, data: &[u8]) -> Result<Self, anyhow::Error> {
        if data.len() != Header::LENGTH {
            return Err(anyhow!(
                "Incorrect length: Expecting: {}, got: {}",
                Header::LENGTH,
                data.len()
            ));
        }

        // Validation only.
        MagicNumber::try_from(data[Self::MAGIC_NUMBER])?;

        let their_network = Network::try_from(data[Self::NETWORK])?;
        if their_network != state.network() {
            return Err(anyhow!(
                "Network mismatch: We're on {:?}, they're on {:?}",
                state.network(),
                their_network
            ));
        }

        // TODO: Check versions (work out what each field means exactly)

        let message_type = MessageType::try_from(data[Self::MESSAGE_TYPE])?;
        let flags = Flags::try_from(&data[Self::FLAGS..Self::FLAGS + Flags::LENGTH])?;

        Ok(Header::new(state.network(), message_type, flags))
    }

    fn len() -> usize {
        Header::LENGTH
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
pub enum Network {
    Test = 0x41,
    Beta = 0x42,
    Live = 0x43,
}

impl TryFrom<u8> for Network {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        use Network::*;
        Ok(match v {
            0x41 => Test,
            0x42 => Beta,
            0x43 => Live,
            v => return Err(anyhow!("Unknown network: {} ({:X})", v, v)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Version {
    Current = 18,
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
    NodeIdHandshake = 10,
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
            10 => NodeIdHandshake,
            11 => BulkPullAccount,
            12 => TelemetryReq,
            13 => TelemetryAck,
            v => return Err(anyhow!("Unknown message type: {}", v)),
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Flags([u8; 2]);

impl Flags {
    const LENGTH: usize = 2;

    const QUERY: usize = 0;
    const RESPONSE: usize = 1;

    pub fn new() -> Self {
        Self([0, 0])
    }

    pub fn set_query(&mut self, v: bool) -> &mut Self {
        self.mut_bits().set(Self::QUERY, v);
        self
    }

    pub fn is_query(&self) -> bool {
        self.bits()[Self::QUERY]
    }

    pub fn set_response(&mut self, v: bool) -> &mut Self {
        self.mut_bits().set(Self::RESPONSE, v);
        self
    }

    pub fn is_response(&self) -> bool {
        self.bits()[Self::RESPONSE]
    }

    fn bits(&self) -> &BitSlice<Lsb0, u8> {
        self.0.view_bits()
    }

    fn mut_bits(&mut self) -> &mut BitSlice<Lsb0, u8> {
        self.0.view_bits_mut()
    }
}

impl std::fmt::Debug for Flags {
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

impl TryFrom<&[u8]> for Flags {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != Self::LENGTH {
            // Probably a coding error rather than external input.
            return Err(anyhow!(
                "Invalid length: Got: {} Expected: {}",
                value.len(),
                Self::LENGTH,
            ));
        }

        let mut s = Self::new();
        s.0[0] = value[0];
        s.0[1] = value[1];
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    #[test]
    fn serialize() {
        let state = State::new(Network::Live);

        let mut flags = Flags::new();
        flags.set_query(true);
        flags.set_response(true);

        let mut h1 = Header::new(state.network(), MessageType::Keepalive, flags);
        let s = h1.serialize();
        assert_eq!(s.len(), Header::LENGTH);
        assert_eq!(s, vec![0x52, 0x43, 18, 18, 18, 2, 3, 0]);

        let h2 = Header::deserialize(&state, &s).unwrap();
        assert_eq!(h1, h2);
    }

    fn assert_contains_err<T: Debug>(result: anyhow::Result<T>, s: &str) {
        let x = result.unwrap_err().to_string();
        assert!(x.contains(s), x);
    }

    #[test]
    fn bad_length() {
        let state = State::new(Network::Live);
        let err = "Incorrect length";
        let s = vec![];
        assert_contains_err(Header::deserialize(&state, &s), err);
        let s = vec![0xFF, 0x43, 18, 18, 18, 2, 3, 0, 0xFF];
        assert_contains_err(Header::deserialize(&state, &s), err);
    }

    #[test]
    fn bad_magic() {
        let state = State::new(Network::Live);
        let s = vec![0xFF, 0x43, 18, 18, 18, 2, 3, 0];
        assert_contains_err(Header::deserialize(&state, &s), "magic number");
    }

    #[test]
    fn bad_network() {
        let state = State::new(Network::Test);
        let s = vec![0x52, 0x43, 18, 18, 18, 2, 3, 0];
        assert_contains_err(Header::deserialize(&state, &s), "Network mismatch");
    }

    #[test]
    fn bad_message_type() {
        let state = State::new(Network::Live);
        let s = vec![0x52, 0x43, 18, 18, 18, 100, 3, 0];
        assert_contains_err(Header::deserialize(&state, &s), "message type");
    }
}
