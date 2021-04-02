mod account_balance;
mod account_history;

use crate::rpc::client::account_balance::AccountBalanceRequest;
use crate::rpc::client::account_history::AccountHistoryRequest;
use crate::{Error, Result};
use clap::Clap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::debug;

trait RPCRequest {
    fn action(&self) -> &str;
}

#[derive(Debug, Serialize)]
pub struct Request<'a, T> {
    action: &'a str,

    #[serde(flatten)]
    data: &'a T,
}

impl<'a, T> Request<'a, T> {
    pub fn new(action: &'a str, data: &'a T) -> Self {
        Self { action, data }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Error(RPCError),
    Success(T),
}

#[derive(Debug, Deserialize)]
pub struct RPCError {
    error: String,
}

pub struct Client {
    url: String,
    authorization: Option<String>,
}

impl Client {
    pub fn new<S: Into<String>>(url: S) -> Self {
        let url = url.into();
        Self {
            url,
            authorization: None,
        }
    }

    pub fn authorization<S: Into<String>>(&mut self, auth: S) {
        self.authorization = Some(auth.into());
    }

    async fn rpc<S, R>(&self, request: &S) -> Result<R>
    where
        S: Sized + Serialize + RPCRequest,
        R: Sized + DeserializeOwned + Debug,
    {
        let action = request.action();
        let client = reqwest::Client::new();

        let body = Request::new(action, request);
        let body = serde_json::to_string(&body).expect("Could not serialize request");
        debug!("SEND: {}", body);

        let mut request = client.post(&self.url);
        if let Some(auth) = &self.authorization {
            request = request.header("Authorization", auth);
        }
        let res = request
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await?;
        let text = res.text().await?;
        debug!("RECV: {}", text);
        let res =
            serde_json::from_str::<Response<R>>(&text).map_err(|err| Error::BadRPCResponse {
                err,
                response: text,
            })?;
        match res {
            Response::Success(res) => Ok(res),
            Response::Error(err) => Err(Error::RPCError(err.error)),
        }
    }
}

#[derive(Clap)]
pub(crate) struct RPCClientOpts {
    #[clap(
        long,
        short,
        default_value = "http://localhost:7076",
        env = "FEELESS_RPC_URL"
    )]
    host: String,

    #[clap(long, short, env = "FEELESS_RPC_AUTH")]
    auth: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    AccountBalance(AccountBalanceRequest),
    AccountHistory(AccountHistoryRequest),
}

impl RPCClientOpts {
    pub(crate) async fn handle(&self) -> Result<()> {
        let mut client = Client::new(&self.host);
        if let Some(a) = &self.auth {
            client.authorization(a);
        }

        match &self.command {
            Command::AccountBalance(c) => show(c.call(&client).await?)?,
            Command::AccountHistory(c) => show(c.call(&client).await?)?,
        };
        Ok(())
    }
}

fn show<T>(response: T) -> Result<()>
where
    T: Serialize,
{
    println!(
        "{}",
        serde_json::to_string(&response).expect("Could not serialize")
    );
    Ok(())
}
