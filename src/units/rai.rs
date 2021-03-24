use super::{MicroNano, Nano};
use crate::units::{Cents, UnboundedRai};
use crate::{expect_len, to_hex};
use bigdecimal::BigDecimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;
use crate::FeelessError;

/// Special bounded container for the smallest unit, rai.
///
/// Can not contain values outside of `0` to [u128::MAX]. To get around this, use [UnboundedRai] or
/// one of the other denominations: [Nano], [Cents], [MicroNano].
///
/// ```
/// use feeless::Rai;
///
/// fn main() -> anyhow::Result<()> {
/// use feeless::units::Nano;
///     let rai = Rai::new(1000000000000000000000000000000u128);
///     assert_eq!(rai.to_nano(), Nano::new(1));
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rai(pub(crate) u128);

impl Rai {
    pub(crate) const LEN: usize = 16;

    /// Create a new [Rai] instance. The value must be [Into<u128>]. This might change to something
    /// more flexible soon!
    pub fn new<T: Into<u128>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn from_hex(s: &str) -> Result<Self, FeelessError> {
        expect_len(s.len(), Rai::LEN * 2, "Hex rai")?;
        let vec = hex::decode(s.as_bytes())?;
        Ok(Rai::try_from(vec.as_slice())?)
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn max() -> Self {
        Self(u128::MAX)
    }

    pub fn to_nano(&self) -> Nano {
        Nano::from(self)
    }

    pub fn to_cents(&self) -> Cents {
        Cents::from(self)
    }

    pub fn to_micro_nano(&self) -> MicroNano {
        MicroNano::from(self)
    }

    pub fn to_unbounded(&self) -> UnboundedRai {
        UnboundedRai::from(self)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    pub fn to_hex_string(&self) -> String {
        to_hex(self.0.to_be_bytes().as_ref())
    }

    pub fn to_u128(&self) -> u128 {
        self.0
    }

    pub fn to_big_decimal(&self) -> BigDecimal {
        // TODO: Don't know why from_u128() doesn't work.
        BigDecimal::from_str(&self.0.to_string()).unwrap()
    }

    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Rai::from)
    }

    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Rai::from)
    }
}

impl FromStr for Rai {
    type Err = FeelessError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u128::from_str(s)?))
    }
}

/// This serializer and deserializer are for strings with decimal numbers. See serialize_to_hex
/// and deserialize_from_hex if you expect your strings to be hex.
impl Serialize for Rai {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Rai {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Rai::from_str(s).map_err(de::Error::custom)?)
    }
}

pub fn serialize_to_hex<S>(
    rai: &Rai,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
{
    serializer.serialize_str(rai.to_hex_string().as_str())
}

pub fn deserialize_from_hex<'de, D>(deserializer: D) -> Result<Rai, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(Rai::from_hex(s).map_err(de::Error::custom)?)
}

impl Display for Rai {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u128> for Rai {
    fn from(v: u128) -> Self {
        Rai(v)
    }
}

impl TryFrom<&[u8]> for Rai {
    type Error = FeelessError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        expect_len(value.len(), Self::LEN, "Raw")?;
        let mut b = [0u8; 16];
        b.copy_from_slice(value);
        let amount = u128::from_be_bytes(b);
        Ok(Rai(amount))
    }
}

impl TryFrom<&BigDecimal> for Rai {
    type Error = FeelessError;

    /// Convert from BigDecimal into Rai, removing any fraction.
    ///
    /// It's up to the caller to round to a whole number beforehand.
    ///
    /// One Rai is monetarily insignificant, but if you're using fractions and trying to encode
    /// data this might bite you!
    fn try_from(value: &BigDecimal) -> Result<Self, Self::Error> {
        // Remove decimals.
        let value = value.with_scale(0);
        // TODO: Don't use strings here.
        // TODO: from_u128 seems broken so we're using strings.
        Self::from_str(value.to_string().as_str())
    }
}

impl PartialOrd for Rai {
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

impl PartialOrd<u128> for Rai {
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

impl PartialEq<u128> for Rai {
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
        assert_eq!(Rai::zero().to_string(), "0");
        assert_eq!(
            Rai::from_str("98765432100123456789").unwrap().to_string(),
            "98765432100123456789"
        );
    }

    #[test]
    fn convert_from_rai() {
        let one_rai = Rai::from(1u128);
        assert_eq!(one_rai.to_string(), "1");
        assert_eq!(
            one_rai.to_nano().to_string(),
            "0.000000000000000000000000000001"
        );
        assert_eq!(
            one_rai.to_micro_nano().to_string(),
            "0.000000000000000000000001"
        );

        assert_eq!(
            MicroNano::new(1).to_rai().unwrap(),
            Rai::from_str("1000000000000000000000000").unwrap()
        );

        assert_eq!(
            Nano::new(1).to_rai().unwrap(),
            Rai::from_str("1000000000000000000000000000000").unwrap()
        );

        let max_rai = Rai::from_str("340282366920938463463374607431768211455").unwrap();
        assert_eq!(
            max_rai.to_string(),
            "340282366920938463463374607431768211455"
        );
        assert_eq!(
            max_rai.to_micro_nano().to_string(),
            "340282366920938.463463374607431768211455"
        );
        assert_eq!(
            max_rai.to_nano().to_string(),
            "340282366.920938463463374607431768211455"
        );
    }

    #[test]
    fn convert_to_rai() {
        assert_eq!(
            MicroNano::new(1).to_rai().unwrap().to_string(),
            "1000000000000000000000000"
        );
        assert_eq!(
            Nano::new(1).to_rai().unwrap().to_string(),
            "1000000000000000000000000000000"
        );
    }

    #[test]
    fn eq() {
        assert_eq!(
            Nano::new(1).to_rai().unwrap(),
            Rai::new(1000000000000000000000000000000u128)
        );
    }

    #[test]
    fn serialize() {
        let rai1 = Nano::new(1).to_rai().unwrap();
        let bytes = rai1.to_vec();
        let rai2 = Rai::try_from(bytes.as_slice()).unwrap();
        assert_eq!(rai1, rai2);
    }

    #[test]
    fn decimal_json() -> anyhow::Result<()> {
        let rai = Nano::new(1).to_rai().unwrap();
        let json = serde_json::to_string(&rai).unwrap();
        assert_eq!(json, r#""1000000000000000000000000000000""#);
        assert_eq!(serde_json::from_str::<Rai>(&json)?, rai);
        Ok(())
    }

    #[test]
    fn hex_json() {
        #[derive(Serialize, Deserialize)]
        struct HexRaw {
            #[serde(
                serialize_with = "serialize_to_hex",
                deserialize_with = "deserialize_from_hex"
            )]
            hex_rai: Rai,
        }
        let hex_rai = HexRaw {
            hex_rai: Nano::new(1).to_rai().unwrap(),
        };
        let json = serde_json::to_string(&hex_rai).unwrap();
        assert_eq!(json, r#"{"hex_rai":"0000000C9F2C9CD04674EDEA40000000"}"#);
        assert_eq!(
            serde_json::from_str::<HexRaw>(&json).unwrap().hex_rai,
            hex_rai.hex_rai
        );
    }

    #[test]
    fn negative_unbounded() {
        let mut v = Rai::zero().to_unbounded();
        v -= UnboundedRai::new(1);
    }
}
