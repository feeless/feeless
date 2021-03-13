use crate::{expect_len, to_hex};
use anyhow::Context;
use bigdecimal::BigDecimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

use std::str::FromStr;

const RAW_TO_MNANO: u128 = 1_000_000_000_000_000_000_000_000_000_000;
const RAW_TO_NANO: u128 = 1_000_000_000_000_000_000_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Raw(u128);

impl Raw {
    pub const LEN: usize = 16;

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn max() -> Self {
        Self(u128::MAX)
    }

    pub fn from_raw<T: Into<u128>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn from_nano<T: Into<u128>>(v: T) -> Self {
        Self(v.into() * RAW_TO_NANO)
    }

    pub fn from_mnano<T: Into<u128>>(v: T) -> Self {
        Self(v.into() * RAW_TO_MNANO)
    }

    pub fn from_raw_str(v: &str) -> anyhow::Result<Self> {
        Ok(Self(u128::from_str(v)?))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    pub fn to_raw_string(&self) -> String {
        self.0.to_string()
    }

    pub fn to_hex_string(&self) -> String {
        to_hex(self.0.to_be_bytes().as_ref())
    }

    pub fn to_raw_u128(&self) -> u128 {
        self.0
    }

    pub fn to_bigdecimal(&self) -> BigDecimal {
        // TODO: Don't know why from_u128() doesn't work.
        BigDecimal::from_str(&self.0.to_string()).unwrap()
    }

    pub fn to_nano_bigdecimal(&self) -> BigDecimal {
        // TODO: Don't know why from_u128() doesn't work.
        self.to_bigdecimal() / BigDecimal::from_str(&RAW_TO_NANO.to_string()).unwrap()
    }

    pub fn to_mnano_bigdecimal(&self) -> BigDecimal {
        // TODO: Don't know why from_u128() doesn't work.
        self.to_bigdecimal() / BigDecimal::from_str(&RAW_TO_MNANO.to_string()).unwrap()
    }

    pub fn to_nano_string(&self) -> String {
        self.to_nano_bigdecimal().to_string()
    }

    pub fn to_mnano_string(&self) -> String {
        self.to_mnano_bigdecimal().to_string()
    }

    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Raw::from)
    }

    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Raw::from)
    }
}

/// This serializer and deserializer are for strings with decimal numbers. See serialize_to_hex
/// and deserialize_from_hex if you expect your strings to be hex.
impl Serialize for Raw {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_raw_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Raw {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Raw::from_raw_str(s).map_err(de::Error::custom)?)
    }
}

pub fn serialize_to_hex<S>(
    raw: &Raw,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
{
    serializer.serialize_str(raw.to_hex_string().as_str())
}

pub fn deserialize_from_hex<'de, D>(deserializer: D) -> Result<Raw, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(Raw::from_str(s).map_err(de::Error::custom)?)
}

impl Display for Raw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_raw_string())
    }
}

impl From<u128> for Raw {
    fn from(v: u128) -> Self {
        Raw(v)
    }
}

impl TryFrom<&[u8]> for Raw {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Raw")?;
        let mut b = [0u8; 16];
        b.copy_from_slice(value);
        let amount = u128::from_be_bytes(b);
        Ok(Raw(amount))
    }
}

impl FromStr for Raw {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        expect_len(s.len(), Raw::LEN * 2, "Hex raw")?;
        let vec = hex::decode(s.as_bytes()).context("Decoding hex raw")?;
        Ok(Raw::try_from(vec.as_slice())?)
    }
}

impl PartialOrd for Raw {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }

    fn lt(&self, other: &Self) -> bool {
        self.0.lt(&other.0)
    }

    fn le(&self, other: &Self) -> bool {
        self.0.le(&other.0)
    }

    fn gt(&self, other: &Self) -> bool {
        self.0.gt(&other.0)
    }

    fn ge(&self, other: &Self) -> bool {
        self.0.ge(&other.0)
    }
}

impl PartialOrd<u128> for Raw {
    fn partial_cmp(&self, other: &u128) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }

    fn lt(&self, other: &u128) -> bool {
        self.0.lt(other)
    }

    fn le(&self, other: &u128) -> bool {
        self.0.le(other)
    }

    fn gt(&self, other: &u128) -> bool {
        self.0.gt(other)
    }

    fn ge(&self, other: &u128) -> bool {
        self.0.ge(other)
    }
}

impl PartialEq<u128> for Raw {
    fn eq(&self, other: &u128) -> bool {
        self.0.eq(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn display() {
        assert_eq!(Raw::zero().to_string(), "0");
        assert_eq!(
            Raw::from_raw_str("98765432100123456789")
                .unwrap()
                .to_string(),
            "98765432100123456789"
        );
    }

    #[test]
    fn convert_from_raw() {
        let one_raw = Raw::from(1u128);
        assert_eq!(one_raw.to_raw_string(), "1");
        assert_eq!(one_raw.to_nano_string(), "0.000000000000000000000001");
        assert_eq!(
            one_raw.to_mnano_string(),
            "0.000000000000000000000000000001"
        );

        assert_eq!(
            Raw::from_nano(1u32),
            Raw::from_raw_str("1000000000000000000000000").unwrap()
        );
        assert_eq!(
            Raw::from_mnano(1u128),
            Raw::from_raw_str("1000000000000000000000000000000").unwrap()
        );

        let max_raw = Raw::from_raw_str("340282366920938463463374607431768211455").unwrap();
        assert_eq!(
            max_raw.to_raw_string(),
            "340282366920938463463374607431768211455"
        );
        assert_eq!(
            max_raw.to_nano_string(),
            "340282366920938.463463374607431768211455"
        );
        assert_eq!(
            max_raw.to_mnano_string(),
            "340282366.920938463463374607431768211455"
        );
    }

    #[test]
    fn convert_to_raw() {
        assert_eq!(
            Raw::from_nano(1u128).to_raw_string(),
            "1000000000000000000000000"
        );
        assert_eq!(
            Raw::from_mnano(1u128).to_raw_string(),
            "1000000000000000000000000000000"
        );
    }

    #[test]
    fn eq() {
        assert_eq!(
            Raw::from_mnano(1u128),
            Raw::from_raw(1000000000000000000000000000000u128)
        );
    }

    #[test]
    fn serialize() {
        let raw1 = Raw::from_mnano(1u128);
        let bytes = raw1.to_vec();
        let raw2 = Raw::try_from(bytes.as_slice()).unwrap();
        assert_eq!(raw1, raw2);
    }

    #[test]
    fn decimal_json() -> anyhow::Result<()> {
        let raw = Raw::from_mnano(1u128);
        let json = serde_json::to_string(&raw).unwrap();
        assert_eq!(json, r#""1000000000000000000000000000000""#);
        assert_eq!(serde_json::from_str::<Raw>(&json)?, raw);
        Ok(())
    }

    #[test]
    fn hex_json() -> anyhow::Result<()> {
        #[derive(Serialize, Deserialize)]
        struct HexRaw {
            #[serde(
                serialize_with = "serialize_to_hex",
                deserialize_with = "deserialize_from_hex"
            )]
            hex_raw: Raw,
        }
        let hex_raw = HexRaw {
            hex_raw: Raw::from_mnano(1u128),
        };
        let json = serde_json::to_string(&hex_raw).unwrap();
        assert_eq!(json, r#"{"hex_raw":"0000000C9F2C9CD04674EDEA40000000"}"#);
        assert_eq!(
            serde_json::from_str::<HexRaw>(&json)?.hex_raw,
            hex_raw.hex_raw
        );
        Ok(())
    }
}
