mod account_balance;
mod account_block_count;
mod account_get;
mod account_history;
mod account_info;
mod account_key;
mod account_representative;
mod account_weight;
mod accounts_balances;
mod accounts_frontiers;
mod accounts_pending;
mod active_difficulty;
mod available_supply;
mod block_account;
mod block_confirm;
mod block_count;
mod block_create;
mod block_info;
mod peers;
mod process;
mod work_validate;

use crate::node::NodeCommandSender;
use crate::Result;
pub use account_balance::{AccountBalanceRequest, AccountBalanceResponse};
pub use account_block_count::{AccountBlockCountRequest, AccountBlockCountResponse};
pub use account_get::{AccountGetRequest, AccountGetResponse};
pub use account_history::{AccountHistoryEntry, AccountHistoryRequest, AccountHistoryResponse};
pub use account_info::{AccountInfoRequest, AccountInfoResponse};
pub use account_key::{AccountKeyRequest, AccountKeyResponse};
pub use account_representative::{AccountRepresentativeRequest, AccountRepresentativeResponse};
pub use account_weight::{AccountWeightRequest, AccountWeightResponse};
pub use accounts_balances::{AccountsBalancesRequest, AccountsBalancesResponse};
pub use accounts_frontiers::{AccountsFrontiersRequest, AccountsFrontiersResponse};
pub use accounts_pending::{AccountsPendingRequest, AccountsPendingResponse};
pub use active_difficulty::{ActiveDifficultyRequest, ActiveDifficultyResponse};
use async_trait::async_trait;
pub use available_supply::{AvailableSupplyRequest, AvailableSupplyResponse};
pub use block_account::{BlockAccountRequest, BlockAccountResponse};
pub use block_confirm::{BlockConfirmRequest, BlockConfirmResponse};
pub use block_count::{BlockCountRequest, BlockCountResponse};
pub use block_create::{BlockCreateRequest, BlockCreateResponse};
pub use block_info::{BlockInfoRequest, BlockInfoResponse};
use clap::Clap;
pub use peers::{DetailedPeerInfo, Peers, PeersRequest, PeersResponse};
pub use process::{ProcessRequest, ProcessResponse};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

pub use work_validate::{WorkValidateRequest, WorkValidateResponse};

#[async_trait]
pub trait NodeHandler {
    type Response: Serialize;

    async fn handle(&self, node_tx: NodeCommandSender) -> Result<Self::Response>;
}

#[derive(Debug, Clap, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RpcCommand {
    AccountBalance(AccountBalanceRequest),
    AccountHistory(AccountHistoryRequest),
    AccountInfo(AccountInfoRequest),
    AccountsPending(AccountsPendingRequest),
    ActiveDifficulty(ActiveDifficultyRequest),
    AccountBlockCount(AccountBlockCountRequest),
    AccountGet(AccountGetRequest),
    AccountKey(AccountKeyRequest),
    AccountRepresentative(AccountRepresentativeRequest),
    AccountsBalances(AccountsBalancesRequest),
    AccountWeight(AccountWeightRequest),
    AccountsFrontiers(AccountsFrontiersRequest),
    AvailableSupply(AvailableSupplyRequest),
    BlockAccount(BlockAccountRequest),
    BlockCount(BlockCountRequest),
    BlockCreate(BlockCreateRequest),
    BlockInfo(BlockInfoRequest),
    BlockConfirm(BlockConfirmRequest),
    Peers(PeersRequest),
    Process(ProcessRequest),
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

pub(crate) fn from_str_option<'de, T, D>(
    deserializer: D,
) -> std::result::Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s)
        .map_err(de::Error::custom)
        .map(|res| Some(res))
}

pub fn as_str<V, S>(v: &V, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Display,
{
    serializer.serialize_str(v.to_string().as_str())
}

pub fn as_str_option<V, S>(v: &Option<V>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Display,
{
    match v {
        Some(a) => serializer.serialize_str(a.to_string().as_str()),
        None => serializer.serialize_str("".to_string().as_str()),
    }
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
