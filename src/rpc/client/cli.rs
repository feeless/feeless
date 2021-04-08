use crate::rpc::calls::Command;
use crate::rpc::client::{RPCClient, RPCRequest};
use clap::Clap;
use colored_json::ToColoredJson;
use serde::Serialize;

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

impl RPCClientOpts {
    pub(crate) async fn handle(&self) -> crate::Result<()> {
        match &self.command {
            Command::AccountBalance(c) => self.show(c).await?,
            Command::AccountHistory(c) => self.show(c).await?,
            Command::AccountInfo(c) => self.show(c).await?,
            Command::ActiveDifficulty(c) => self.show(c).await?,
            Command::BlockInfo(c) => self.show(c).await?,
            Command::WorkValidate(c) => self.show(c).await?,
            Command::AccountBlockCount(c) => self.show(c).await?,
            Command::AccountGet(c) => self.show(c).await?,
            Command::AccountKey(c) => self.show(c).await?,
            Command::AccountRepresentative(c) => self.show(c).await?,
            Command::AccountWeight(c) => self.show(c).await?,
        };
        Ok(())
    }

    async fn show<T>(&self, request: T) -> crate::Result<()>
    where
        T: Serialize + RPCRequest,
    {
        let mut client = RPCClient::new(&self.host);
        if let Some(a) = &self.auth {
            client.authorization(a);
        }

        let response = request.call(&client).await?;
        println!(
            "{}",
            serde_json::to_string_pretty(&response)
                .expect("Could not serialize")
                .to_colored_json_auto()
                .expect("Could not colorize")
        );
        Ok(())
    }
}
