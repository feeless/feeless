use anyhow::anyhow;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::io::Read;
use std::str::FromStr;

mod address;
pub mod convert;
mod phrase;
mod private;
mod public;
mod seed;

pub use address::Address;
pub use phrase::Phrase;
pub use private::Private;
pub use public::Public;
pub use seed::Seed;

/// The a `T` or the String "-" if reading from stdin.
///
/// Use `resolve()` to turn the enum into `T` by maybe reading from stdin.
#[derive(Copy, Clone)]
pub enum StringOrStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    String(T),
    Stdin,
}

impl<T> StringOrStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    /// Resolve `T` by reading from stdin if necessary.
    pub fn resolve(self) -> anyhow::Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        match self {
            StringOrStdin::String(t) => Ok(t),
            StringOrStdin::Stdin => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                Ok(T::from_str(buffer.trim())
                    .map_err(|e| anyhow!("Conversion from string failed: {:?}", e))?)
            }
        }
    }
}

impl<T> FromStr for StringOrStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Err = anyhow::Error;

    // This wasn't done in one step because I think clap calls from_str twice, and the second time
    // around stdin is empty.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_ref() {
            "-" => Ok(StringOrStdin::Stdin),
            x => match T::from_str(x) {
                Ok(x) => Ok(StringOrStdin::String(x)),
                Err(e) => Err(anyhow!("Could not parse string: {:?}", e)),
            },
        }
    }
}
