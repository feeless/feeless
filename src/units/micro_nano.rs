use super::rai::Rai;
use anyhow::anyhow;
use bigdecimal::{BigDecimal, FromPrimitive, Signed};

#[derive(Debug, Clone)]
pub struct MicroNano(BigDecimal);

impl MicroNano {
    const TO_RAI: u128 = 1_000_000_000_000_000_000_000_000;

    pub fn new<T: Into<BigDecimal>>(v: T) -> Self {
        Self(v.into())
    }

    pub fn to_rai(&self) -> anyhow::Result<Rai> {
        // TODO: unwrap ok here?
        // let v = check_overflow(&self.0 * BigDecimal::from_u128(Self::TO_RAI).unwrap())?;

        todo!()
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
