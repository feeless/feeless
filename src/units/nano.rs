use super::rai::Rai;
use anyhow::anyhow;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use once_cell::sync::Lazy;
use std::str::FromStr;

static TO_RAI: Lazy<BigDecimal> = Lazy::new(|| {
    // For some reason from_u128 fails with `None`.
    BigDecimal::from_str("1_000_000_000_000_000_000_000_000_000_000").unwrap()
});

pub struct Nano(BigDecimal);

/// Nano (10<sup>30</sup> rai). Overflow and negative numbers allowed, except when converting to [Rai].
impl Nano {
    pub fn new<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn to_rai(&self) -> anyhow::Result<Rai> {
        let bd = &self.0 * &*TO_RAI;
        let v = bd
            .to_u128()
            .ok_or_else(|| anyhow!("{} is out of range of 0..u128::MAX", bd))?;
        Ok(Rai::new(v))
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
        Self(BigDecimal::from_u128(rai.0).unwrap() / &*TO_RAI)
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
        assert_eq!(
            u128::MAX.to_string(),
            BigDecimal::from_u128(u128::MAX).unwrap().to_string()
        );
    }
}
