mod account_balance;
mod account_history;
mod account_info;
mod active_difficulty;
mod block_info;
mod work_validate;

pub use account_balance::{AccountBalanceRequest, AccountBalanceResponse};
pub use account_history::{AccountHistoryEntry, AccountHistoryRequest, AccountHistoryResponse};
pub use account_info::{AccountInfoRequest, AccountInfoResponse};
pub use active_difficulty::{ActiveDifficultyRequest, ActiveDifficultyResponse};
pub use block_info::{BlockInfoRequest, BlockInfoResponse};
use clap::Clap;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
pub use work_validate::{WorkValidateRequest, WorkValidateResponse};

#[derive(Debug, Clap, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    AccountBalance(AccountBalanceRequest),
    AccountHistory(AccountHistoryRequest),
    AccountInfo(AccountInfoRequest),
    ActiveDifficulty(ActiveDifficultyRequest),
    BlockInfo(BlockInfoRequest),
    WorkValidate(WorkValidateRequest),
}

pub(crate) fn from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

pub fn as_str<V, S>(v: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Display,
{
    serializer.serialize_str(v.to_string().as_str())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
