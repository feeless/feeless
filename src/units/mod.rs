//! Units of Nano, i.e.
//! [Rai],
//! [Nano] (10<sup>30</sup>),
//! [Cents] (10<sup>22</sup>),
//! [MicroNano] (10<sup>24</sup>).
//!
//! Please note these are different from the currently used units, using the [proposed
//! new currency units pull request](https://github.com/nanocurrency/nano-docs/pull/466).
//!
//! [Rai] acts differently than the other units in this module, as its internal type is [u128].
//! This means it can not be a value outside of that, e.g. negative numbers. To get around this,
//! use [UnboundedRai], which internally uses [BigDecimal]. [Nano], [Cents], [MicroNano] and
//! [UnboundedRai] all use [BigDecimal] internally.
//!
//! Example:
//! ```
//! use feeless::units::{Nano, Cents};
//! use std::convert::TryFrom;
//! use std::str::FromStr;
//!
//! # fn main() -> anyhow::Result<()> {
//! // One Nano.
//! let nano = Nano::new(1);
//!
//! // Convert cents.
//! let cents = nano.to_cents();
//! assert_eq!(cents, Cents::new(100));
//!
//! // Works with arithmetic.
//! let cents = (cents * Cents::new(-10) + Cents::new(1)) / Cents::new(1000);
//! // Can parse fractional strings.
//! assert_eq!(cents, Cents::from_str("-0.999")?);
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with floats (f32, f64)
//! The general recommendation is to never use floats when dealing with money due to inaccuracies
//! with floating point precision. You can however do it with [BigDecimal]--see the example below.
//!
//! Ideally if your API doesn't support [BigDecimal], it might be better to convert between
//! [String] and [BigDecimal] to make sure there are no rounding or floating point inaccuracies.
//! ```
//! use feeless::units::Cents;
//! use bigdecimal::{BigDecimal, FromPrimitive};
//! use std::str::FromStr;
//!
//! fn main() -> anyhow::Result<()> {
//!     // If you really need to load from a float, use BigDecimal.
//!     let big = BigDecimal::from_f64(1231239999999999.1).unwrap();
//!     let cents = Cents::new(big);
//!
//!     // Convert to float.
//!     assert_eq!(cents.to_f64(), 1231239999999999.1);
//!
//!     // Better
//!     let big = BigDecimal::from_str("9999999999.1").unwrap();
//!     let cents = Cents::new(big);
//!     assert_eq!(cents.to_string(), "9999999999.1");
//!
//!     Ok(())
//! }
//! ```
pub(crate) mod rai;

use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
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
                    let value = 10u128.pow($multiplier);
                    // For some reason from_u128 fails with `None`.
                    BigDecimal::from_str(value.to_string().as_str()).unwrap()
                });
                multiplier
            }

            pub fn new<T: Into<BigDecimal>>(v: T) -> Self {
                Self(v.into())
            }

            /// Use rai denomination when creating this unit.
            ///
            /// e.g.
            /// ```
            /// # use feeless::units::MicroNano;
            /// # use bigdecimal::BigDecimal;
            /// # use std::str::FromStr;
            /// # fn main() -> anyhow::Result<()> {
            /// let cents = MicroNano::new_with_rai(BigDecimal::from_str("1_000_000_000_000_000_000_000_000").unwrap());
            /// assert_eq!(cents, MicroNano::new(1));
            /// # Ok(())
            /// # }
            /// ```
            pub fn new_with_rai<T: Into<BigDecimal>>(v: T) -> Self {
                Self(v.into() / &*Self::lazy_multiplier())
            }

            pub fn to_rai(&self) -> anyhow::Result<Rai> {
                Rai::try_from(&self.to_rai_big_decimal())
            }

            pub fn to_unbounded_rai(&self) -> UnboundedRai {
                UnboundedRai::new(self.to_rai_big_decimal())
            }

            pub fn to_micro_nano(&self) -> MicroNano {
                MicroNano::new_with_rai(self.to_rai_big_decimal())
            }

            pub fn to_cents(&self) -> Cents {
                Cents::new_with_rai(self.to_rai_big_decimal())
            }

            pub fn to_nano(&self) -> Nano {
                Nano::new_with_rai(self.to_rai_big_decimal())
            }

            pub fn to_f64(&self) -> f64 {
                // TODO: unwrap ok here?
                self.0.to_f64().unwrap()
            }

            pub fn to_big_decimal(&self) -> &BigDecimal {
                &self.0
            }

            pub fn to_rai_big_decimal(&self) -> BigDecimal {
                &self.0 * &*Self::lazy_multiplier()
            }
        }

        impl ToString for $struct_name {
            fn to_string(&self) -> String {
                self.0.to_string()
            }
        }

        impl FromStr for $struct_name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::new(BigDecimal::from_str(s)?))
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

unit!(Nano, 30);
unit!(Cents, 28);
unit!(MicroNano, 24);
unit!(UnboundedRai, 0);

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::FromPrimitive;

    #[test]
    fn conversions() {
        let rai = Rai::new(1u128)
            .to_nano()
            .to_cents()
            .to_micro_nano()
            .to_rai()
            .unwrap();
        assert_eq!(rai, Rai::new(1u128));

        let nano = Nano::new(1)
            .to_cents()
            .to_cents()
            .to_cents()
            .to_cents() // Yes you can convert from cents to cents. Generated by the macro!
            .to_micro_nano()
            .to_nano();
        assert_eq!(nano, Nano::new(1));
    }

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
