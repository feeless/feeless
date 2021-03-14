pub(crate) mod rai;

use bigdecimal::BigDecimal;
use once_cell::sync::Lazy;
pub use rai::Rai;
use std::convert::TryFrom;
use std::str::FromStr;

/// This macro creates a struct to handle a specific denomination with arithmetic and conversions
/// to/from [Rai].
macro_rules! unit {
    ($struct_name:ident, $multiplier:expr) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $struct_name(BigDecimal);

        impl $struct_name {
            fn lazy_multiplier() -> Lazy<BigDecimal> {
                let multiplier: Lazy<BigDecimal> = Lazy::new(|| {
                    // For some reason from_u128 fails with `None`.
                    BigDecimal::from_str($multiplier.to_string().as_str()).unwrap()
                });
                multiplier
            }

            pub fn new<T: Into<BigDecimal>>(v: T) -> Self {
                Self(v.into())
            }

            pub fn to_rai(&self) -> anyhow::Result<Rai> {
                Rai::try_from(&(&self.0 * &*Self::lazy_multiplier()))
            }
        }

        impl ToString for $struct_name {
            fn to_string(&self) -> String {
                self.0.to_string()
            }
        }

        impl From<Rai> for $struct_name {
            fn from(rai: Rai) -> Self {
                // TODO: unwrap ok here?
                // TODO: from_u128 returns None for some reason...
                let big_dec = BigDecimal::from_str(rai.0.to_string().as_str()).unwrap();
                Self(big_dec / &*Self::lazy_multiplier())
            }
        }

        impl From<&Rai> for $struct_name {
            fn from(rai: &Rai) -> Self {
                Self::from(rai.clone())
            }
        }

        impl std::ops::Add for $struct_name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $struct_name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.0 - rhs.0)
            }
        }

        impl std::ops::Div for $struct_name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.0 / rhs.0)
            }
        }

        impl std::ops::Mul for $struct_name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.0 * rhs.0)
            }
        }

        impl std::ops::AddAssign for $struct_name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl std::ops::SubAssign for $struct_name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl std::ops::MulAssign for $struct_name {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        // TODO: binary assignment operation `/=` cannot be applied to type `bigdecimal::BigDecimal`
        // impl std::ops::DivAssign for $struct_name {
        //     fn div_assign(&mut self, rhs: Self) {
        //         self.0 /= rhs.0;
        //     }
        // }
    };
}

unit!(Nano, 1_000_000_000_000_000_000_000_000_000_000u128);
unit!(Cents, 10_000_000_000_000_000_000_000_000_000u128);
unit!(MicroNano, 1_000_000_000_000_000_000_000_000u128);
unit!(UnboundedRai, 1);

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::FromPrimitive;

    #[test]
    fn negative() {
        assert!(MicroNano::new(-1).to_rai().is_err());
    }

    #[test]
    fn micro_nano_overflow() {
        assert!(MicroNano::new(340282366920938u64).to_rai().is_ok());
        assert!(MicroNano::new(340282366920939u64).to_rai().is_err());

        MicroNano::new(BigDecimal::from_str("340282366920938.4").unwrap())
            .to_rai()
            .unwrap();
        assert!(
            MicroNano::new(BigDecimal::from_str("340282366920938.5").unwrap())
                .to_rai()
                .is_err()
        );
        // Just one rai over the max.
        let d = BigDecimal::from_str("340282366920938.463463374607431768211456").unwrap();
        let result = MicroNano::new(d).to_rai();
        assert!(result.is_err());
    }

    #[test]
    fn nano_overflow() {
        let d = BigDecimal::from_str("340282366.920938463463374607431768211455").unwrap();
        Nano::new(d).to_rai().unwrap();

        // One over the max
        let d = BigDecimal::from_str("340282366.920938463463374607431768211456").unwrap();
        assert!(Nano::new(d).to_rai().is_err());
    }

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
    fn arithmetic() {
        assert_eq!(Nano::new(1) + Nano::new(-2), Nano::new(-1));
        assert_eq!(Nano::new(2) - Nano::new(1), Nano::new(1));
        assert_eq!(Nano::new(10) / Nano::new(2), Nano::new(5));
        assert_eq!(Nano::new(10) * Nano::new(2), Nano::new(20));

        let mut n = Nano::new(1);
        n += Nano::new(2);
        assert_eq!(n, Nano::new(3));
        n -= Nano::new(1);
        assert_eq!(n, Nano::new(2));
        n *= Nano::new(4);
        assert_eq!(n, Nano::new(8));
    }

    #[test]
    fn cents() {
        let nano = Nano::new(1).to_rai().unwrap();
        let cents = nano.to_cents();
        assert_eq!(cents.to_string(), "100");

        let nano = Nano::new(BigDecimal::from_f64(0.01f64).unwrap())
            .to_rai()
            .unwrap();
        let cents = nano.to_cents();
        assert_eq!(cents.to_string(), "1");
    }
}
