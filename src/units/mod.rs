mod macros;
pub(crate) mod micro_nano;
pub(crate) mod nano;
pub(crate) mod rai;

use bigdecimal::BigDecimal;
pub use micro_nano::MicroNano;
pub use nano::Nano;
use once_cell::sync::Lazy;
pub use rai::Rai;
use std::convert::TryFrom;
use std::str::FromStr;

macro_rules! unit {
    ($struct_name:ident, $multiplier:expr) => {
        #[derive(Debug, Clone)]
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
    };
}

unit!(MicroNanoTest, 1_000_000_000_000_000_000_000_000u128);

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::One;

    #[test]
    fn sanity() {
        let a = MicroNanoTest(BigDecimal::one());
        let a = MicroNanoTest::new(0);
        let a = MicroNanoTest::new(1).to_rai().unwrap().to_string();
        dbg!(a);
        let a: MicroNanoTest = Rai::max().into();
        dbg!(a);
    }
}
