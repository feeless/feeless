use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Network version of a node.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Version {
    V18 = 18,
    V19 = 19,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string()[1..].to_owned())
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "18" => Version::V18,
            "19" => Version::V19,
            v => return Err(Error::InvalidVersion(v.into())),
        })
    }
}
