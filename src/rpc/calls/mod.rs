mod account_balance;
mod account_history;
mod account_info;
mod active_difficulty;
mod block_info;
mod work_validate;
mod account_block_count;
mod account_get;
mod account_key;
mod account_representative;
mod account_weight;
mod accounts_balances;
mod accounts_frontiers;
mod available_supply;
mod block_account;
mod block_confirm;
mod block_count;

pub use account_balance::{AccountBalanceRequest, AccountBalanceResponse};
pub use account_history::{AccountHistoryEntry, AccountHistoryRequest, AccountHistoryResponse};
pub use account_info::{AccountInfoRequest, AccountInfoResponse};
pub use active_difficulty::{ActiveDifficultyRequest, ActiveDifficultyResponse};
pub use block_info::{BlockInfoRequest, BlockInfoResponse};
pub use account_block_count::{AccountBlockCountRequest, AccountBlockCountResponse};
pub use account_get::{AccountGetRequest, AccountGetResponse};
pub use account_key::{AccountKeyRequest, AccountKeyResponse};
pub use account_representative::{AccountRepresentativeRequest, AccountRepresentativeResponse};
pub use account_weight::{AccountWeightRequest, AccountWeightResponse};
pub use accounts_balances::{AccountsBalancesRequest, AccountsBalancesResponse};
pub use accounts_frontiers::{AccountsFrontiersRequest, AccountsFrontiersResponse};
pub use available_supply::{AvailableSupplyRequest, AvailableSupplyResponse};
pub use block_account::{BlockAccountRequest, BlockAccountResponse};
pub use block_confirm::{BlockConfirmRequest, BlockConfirmResponse};
pub use block_count::{BlockCountRequest, BlockCountResponse};
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
    AccountBlockCount(AccountBlockCountRequest),
    AccountGet(AccountGetRequest),
    AccountKey(AccountKeyRequest),
    AccountRepresentative(AccountRepresentativeRequest),
    AccountWeight(AccountWeightRequest),
    AccountsBalances(AccountsBalancesRequest),
    AccountsFrontiers(AccountsFrontiersRequest),
    AvailableSupply(AvailableSupplyRequest),
    BlockAccount(BlockAccountRequest),
    BlockConfirm(BlockConfirmRequest),
    BlockCount(BlockCountRequest),
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
