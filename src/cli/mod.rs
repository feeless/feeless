use anyhow::anyhow;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::io::Read;
use std::str::FromStr;

pub mod convert;
pub mod public;

/// Similar to `Option`, except it will be explicitly a Pipe or a value.
#[derive(Copy, Clone)]
pub enum OptionPipe<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    Some(T),
    Pipe,
}

impl<T> OptionPipe<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    /// Resolve `T` by reading from stdin if necessary.
    pub fn resolve(self) -> anyhow::Result<T>
    where
        T: FromStr<Err = anyhow::Error>,
        <T as FromStr>::Err: Debug,
    {
        match self {
            OptionPipe::Some(t) => Ok(t),
            OptionPipe::Pipe => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                T::from_str(buffer.trim())
            }
        }
    }
}

impl<T> FromStr for OptionPipe<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_ref() {
            "-" => Ok(OptionPipe::Pipe),
            x => match T::from_str(x) {
                Ok(x) => Ok(OptionPipe::Some(x)),
                Err(e) => Err(anyhow!("Could not parse string: {:?}", e)),
            },
        }
    }
}
