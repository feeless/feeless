use super::rai::Rai;
use anyhow::anyhow;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use once_cell::sync::Lazy;
use std::convert::TryFrom;
use std::ops::{Add, Div};
use std::str::FromStr;

static TO_RAI: Lazy<BigDecimal> = Lazy::new(|| {
    // For some reason from_u128 fails with `None`.
    BigDecimal::from_str("1_000_000_000_000_000_000_000_000_000_000").unwrap()
});

#[derive(Debug, Clone, PartialEq)]
pub struct Nano(BigDecimal);

/// Nano (10<sup>30</sup> rai). Overflow and negative numbers allowed, except when converting to [Rai].
impl Nano {
    pub fn new<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn to_rai(&self) -> anyhow::Result<Rai> {
        Rai::try_from(&(&self.0 * &*TO_RAI))
    }
}

impl ToString for Nano {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&Rai> for Nano {
    fn from(rai: &Rai) -> Self {
        // TODO: unwrap ok here?
        // TODO: from_u128 returns None for some reason...
        Self(BigDecimal::from_str(rai.0.to_string().as_str()).unwrap() / &*TO_RAI)
    }
}

impl std::ops::Add for Nano {
    type Output = Nano;

    fn add(self, rhs: Self) -> Self::Output {
        Nano::new(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Nano {
    type Output = Nano;

    fn sub(self, rhs: Self) -> Self::Output {
        Nano::new(self.0 - rhs.0)
    }
}

impl std::ops::Div for Nano {
    type Output = Nano;

    fn div(self, rhs: Self) -> Self::Output {
        Nano::new(self.0 / rhs.0)
    }
}

impl std::ops::Mul for Nano {
    type Output = Nano;

    fn mul(self, rhs: Self) -> Self::Output {
        Nano::new(self.0 * rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_something_wrong_with_big_decimal_u128() {
        assert_eq!(
            u64::MAX.to_string(),
            BigDecimal::from_u64(u64::MAX).unwrap().to_string()
        );
        // TODO: This doesn't work
        // assert_eq!(
        //     u128::MAX.to_string(),
        //     BigDecimal::from_u128(u128::MAX).unwrap().to_string()
        // );
    }

    #[test]
    fn overflow() {
        let d = BigDecimal::from_str("340282366.920938463463374607431768211455").unwrap();
        Nano::new(d).to_rai().unwrap();

        // One over the max
        let d = BigDecimal::from_str("340282366.920938463463374607431768211456").unwrap();
        assert!(Nano::new(d).to_rai().is_err());
    }

    #[test]
    fn arithmetic() {
        assert_eq!(Nano::new(1) + Nano::new(-2), Nano::new(-1));
        assert_eq!(Nano::new(2) - Nano::new(1), Nano::new(1));
        assert_eq!(Nano::new(10) / Nano::new(2), Nano::new(5));
        assert_eq!(Nano::new(10) * Nano::new(2), Nano::new(20));
    }
}
