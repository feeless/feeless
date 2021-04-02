mod cli;

use crate::{Error, Result};
use async_trait::async_trait;
pub(crate) use cli::RPCClientOpts;
use serde::de::DeserializeOwned;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use tracing::debug;

#[async_trait]
pub(crate) trait RPCRequest {
    type Response: Serialize;

    fn action(&self) -> &str;
    async fn call(&self, client: &Client) -> Result<Self::Response>;
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

    pub(crate) async fn rpc<S, R>(&self, request: &S) -> Result<R>
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

        // This used to decode into an untagged enum, i.e.
        // `enum Response<T> { Success(T), Error(RPCError) }`
        // When there's an expected field from the RPC response, serde gives a non useful error:
        // `data did not match any variant of untagged enum Response`
        // Related issue: https://github.com/serde-rs/serde/issues/773
        // This code now tries one then the other manually instead of using the enum.
        let result = serde_json::from_str::<R>(&text).map_err(|err| Error::BadRPCResponse {
            err,
            response: text.to_owned(),
        });
        match result {
            Ok(t) => Ok(t),
            Err(err) => {
                match serde_json::from_str::<RPCError>(&text) {
                    Ok(err) => Err(Error::RPCError(err.error)),
                    Err(_) => {
                        // We have an error in both matching R and RPCError, let's return the error
                        // given by from_str::<R>.
                        Err(err)
                    }
                }
            }
        }
    }
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
