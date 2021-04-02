mod account_balance;
mod account_history;
mod account_info;
mod active_difficulty;
mod block_info;

pub use account_balance::{AccountBalanceRequest, AccountBalanceResponse};
pub use account_history::{AccountHistoryEntry, AccountHistoryRequest, AccountHistoryResponse};
pub use account_info::{AccountInfoRequest, AccountInfoResponse};
pub use active_difficulty::{ActiveDifficultyRequest, ActiveDifficultyResponse};
pub use block_info::{BlockInfoRequest, BlockInfoResponse};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

pub(crate) fn from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct AlwaysTrue(bool);

impl Default for AlwaysTrue {
    fn default() -> Self {
        Self(true)
    }
}

impl Deref for AlwaysTrue {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
