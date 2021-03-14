use super::rai::Rai;
use anyhow::anyhow;
use bigdecimal::{BigDecimal, FromPrimitive, Signed};
use once_cell::sync::Lazy;
use std::convert::TryFrom;
use std::str::FromStr;

static TO_RAI: Lazy<BigDecimal> = Lazy::new(|| {
    // For some reason from_u128 fails with `None`.
    BigDecimal::from_str("1_000_000_000_000_000_000_000_000").unwrap()
});

#[derive(Debug, Clone)]
pub struct MicroNano(BigDecimal);

impl MicroNano {
    pub fn new<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn to_rai(&self) -> anyhow::Result<Rai> {
        Rai::try_from(&(&self.0 * &*TO_RAI))
    }
}

// pub fn check_overflow(v: BigDecimal) -> anyhow::Result<BigDecimal> {
//     if v.is_negative() {
//         return Err(anyhow!("Value is negative"));
//     }
//
//     // TODO: unwrap ok here?
//     if v > BigDecimal::from_u128(u128::MAX).unwrap() {
//         return Err(anyhow!("Value is higher than u128::MAX"));
//     }
//
//     Ok(v)
// }
//
impl ToString for MicroNano {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&Rai> for MicroNano {
    fn from(rai: &Rai) -> Self {
        // TODO: unwrap ok here?
        // TODO: from_u128 returns None for some reason...
        Self(BigDecimal::from_str(rai.0.to_string().as_str()).unwrap() / &*TO_RAI)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn negative() {
        assert!(MicroNano::new(-1).to_rai().is_err());
    }
    #[test]
    fn overflow() {
        assert!(MicroNano::new(340282366920938u64).to_rai().is_ok());
        assert!(MicroNano::new(340282366920939u64).to_rai().is_err());

        assert!(
            MicroNano::new(BigDecimal::from_str("340282366920938.4").unwrap())
                .to_rai()
                .is_ok()
        );
        // assert!(MicroNano::new("340282366920938.5").to_rai().is_err());
    }
}
