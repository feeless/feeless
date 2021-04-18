use crate::rpc::calls::RpcCommand;
use crate::rpc::client::{RPCClient, RPCRequest};
use clap::Clap;
use colored_json::ToColoredJson;
use serde::Serialize;

#[derive(Clap)]
pub(crate) struct RPCClientOpts {
    /// The URL of the RPC server.
    #[clap(
        long,
        short,
        default_value = "http://localhost:7076",
        env = "FEELESS_RPC_URL"
    )]
    url: String,

    /// Send a string in the HTTP authorization header.
    #[clap(long, short, env = "FEELESS_RPC_AUTH")]
    auth: Option<String>,

    /// The RPC call to make.
    #[clap(subcommand)]
    command: RpcCommand,
}

impl RPCClientOpts {
    pub(crate) async fn handle(&self) -> crate::Result<()> {
        match &self.command {
            RpcCommand::AccountBalance(c) => self.show(c).await?,
            RpcCommand::AccountBlockCount(c) => self.show(c).await?,
            RpcCommand::AccountGet(c) => self.show(c).await?,
            RpcCommand::AccountHistory(c) => self.show(c).await?,
            RpcCommand::AccountInfo(c) => self.show(c).await?,
            RpcCommand::AccountKey(c) => self.show(c).await?,
            RpcCommand::AccountRepresentative(c) => self.show(c).await?,
            RpcCommand::AccountWeight(c) => self.show(c).await?,
            RpcCommand::AccountsBalances(c) => self.show(c).await?,
            RpcCommand::AccountsFrontiers(c) => self.show(c).await?,
            RpcCommand::AccountsPending(c) => self.show(c).await?,
            RpcCommand::ActiveDifficulty(c) => self.show(c).await?,
            RpcCommand::AvailableSupply(c) => self.show(c).await?,
            RpcCommand::BlockAccount(c) => self.show(c).await?,
            RpcCommand::BlockConfirm(c) => self.show(c).await?,
            RpcCommand::BlockCount(c) => self.show(c).await?,
            RpcCommand::BlockCreate(c) => self.show(c).await?,
            RpcCommand::BlockInfo(c) => self.show(c).await?,
            RpcCommand::Peers(c) => self.show(c).await?,
            RpcCommand::Process(c) => self.show(c).await?,
            RpcCommand::WorkValidate(c) => self.show(c).await?,
        };
        Ok(())
    }

    async fn show<T>(&self, request: T) -> crate::Result<()>
    where
        T: Serialize + RPCRequest,
    {
        let mut client = RPCClient::new(&self.url);
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
