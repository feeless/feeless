use bigdecimal::BigDecimal;
use std::fmt::Display;
use std::str::FromStr;
use bigdecimal::ToPrimitive;
use anyhow::Result;
use serde::{Serialize, Deserialize};

const RAW_TO_NANO: &str = "1_000_000_000_000_000_000_000_000";
const RAW_TO_MNANO: &str = "1_000_000_000_000_000_000_000_000_000_000";

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Raw(BigDecimal);

impl Raw {
    pub fn zero() -> Self {
        Self(BigDecimal::from(0))
    }

    pub fn from_raw<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn from_nano<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into() * BigDecimal::from_str(RAW_TO_NANO).unwrap())
    }

    pub fn from_mnano<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into() * BigDecimal::from_str(RAW_TO_MNANO).unwrap())
    }

    pub fn from_raw_str(v: &str) -> Result<Self> {
        Ok(Self(BigDecimal::from_str(v)?))
    }

    pub fn from_nano_str(v: &str) -> Result<Self> {
        Ok(Self(
            BigDecimal::from_str(v)? * BigDecimal::from_str(RAW_TO_NANO).unwrap(),
        ))
    }

    pub fn from_mnano_str(v: &str) -> Result<Self> {
        Ok(Self(
            BigDecimal::from_str(v)? * BigDecimal::from_str(RAW_TO_MNANO).unwrap(),
        ))
    }

    pub fn to_raw_string(&self) -> String {
        self.0.to_string()
    }

    pub fn to_raw_bigdecimal(&self) -> BigDecimal {
        self.0.clone()
    }

    pub fn to_nano_bigdecimal(&self) -> BigDecimal {
        self.0.clone() / BigDecimal::from_str(RAW_TO_NANO).unwrap()
    }

    pub fn to_mnano_bigdecimal(&self) -> BigDecimal {
        self.0.clone() / BigDecimal::from_str(RAW_TO_MNANO).unwrap()
    }

    pub fn try_to_mnano_i64(&self) -> Option<i64> {
        self.to_mnano_bigdecimal().to_i64()
    }

    pub fn to_nano_string(&self) -> String {
        self.to_nano_bigdecimal().to_string()
    }

    pub fn to_mnano_string(&self) -> String {
        self.to_mnano_bigdecimal().to_string()
    }
}

impl Display for Raw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_raw_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Raw::zero().to_string(), "0");
        assert_eq!(
            Raw::from_raw_str("9876543210.0123456789")
                .unwrap()
                .to_string(),
            "9876543210.0123456789"
        );
    }

    #[test]
    fn convert_from_raw() {
        let one_raw = Raw::from_raw(1);
        assert_eq!(one_raw.to_raw_string(), "1");
        assert_eq!(one_raw.to_nano_string(), "0.000000000000000000000001");
        assert_eq!(
            one_raw.to_mnano_string(),
            "0.000000000000000000000000000001"
        );

        assert_eq!(
            Raw::from_nano(1),
            Raw::from_raw_str("1000000000000000000000000").unwrap()
        );
        assert_eq!(
            Raw::from_mnano(1),
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
            Raw::from_nano_str("1").unwrap().to_raw_string(),
            "1000000000000000000000000"
        );
        assert_eq!(
            Raw::from_mnano_str("1").unwrap().to_raw_string(),
            "1000000000000000000000000000000"
        );
    }

    #[test]
    fn eq() {
        assert_eq!(
            Raw::from_mnano_str("1").unwrap(),
            Raw::from_raw_str("1000000000000000000000000000000").unwrap()
        );
    }
}
