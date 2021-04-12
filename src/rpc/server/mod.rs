mod handlers;

use crate::node::ArcState;
use crate::rpc::client::RPCError;
use crate::rpc::server::handlers::handle_process;
use crate::rpc::{Command, CommandResponse};
use crate::Result;
use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, trace};
use warp::http::StatusCode;
use warp::Filter;

type RpcMessageSender = mpsc::Sender<RPCMessage>;
type RpcMessageResponseSender = oneshot::Sender<CommandResponse>;
type RpcMessageResponseReceiver = oneshot::Receiver<CommandResponse>;

#[derive(Debug)]
pub struct RPCMessage {
    command: Command,
    tx: RpcMessageResponseSender,
}

impl RPCMessage {
    fn new_with_channel(command: Command) -> (Self, RpcMessageResponseReceiver) {
        let (tx, rx) = oneshot::channel();
        (Self { command, tx }, rx)
    }
}

pub struct RPCServer {
    state: ArcState,
    tx: RpcMessageSender,
}

impl RPCServer {
    pub fn new_with_rx(state: ArcState) -> (Self, mpsc::Receiver<RPCMessage>) {
        let (tx, rx) = mpsc::channel::<RPCMessage>(100);
        let s = Self { tx, state };
        (s, rx)
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!("Starting RPC server");
        let rpc = warp::post()
            .and(warp::body::content_length_limit(1024 * 16))
            .and(with_state(self.state.clone()))
            .and(with_tx(self.tx.clone()))
            .and(warp::body::json())
            .and_then(Self::handle);

        // TODO: Configurable
        warp::serve(rpc).run(([127, 0, 0, 1], 7076)).await;
        Ok(())
    }

    async fn handle(
        state: ArcState,
        tx: RpcMessageSender,
        cmd: Command,
    ) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
        trace!("Handling command: {:?}", cmd);
        match cmd {
            // TODO: Example usage
            // Command::ActiveDifficulty(c) => json(&ActiveDifficultyResponse {
            //     multiplier: 1.5,
            //     network_current: Difficulty::new(1),
            //     network_minimum: Difficulty::new(2),
            //     network_receive_current: Difficulty::new(3),
            //     network_receive_minimum: Difficulty::new(4),
            // }),
            Command::Process(c) => json_result(handle_process(state, tx, c).await),
            action => json_result(Ok(RPCError {
                error: format!("The action: {:?} is unhandled", action),
            })),
        }
    }
}

fn with_tx(
    tx: RpcMessageSender,
) -> impl Filter<Extract = (RpcMessageSender,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

fn with_state(
    state: ArcState,
) -> impl Filter<Extract = (ArcState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn json_result<T>(result: Result<T>) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection>
where
    T: Sized + Serialize,
{
    match &result {
        Ok(o) => json(o),
        Err(err) => todo!("{:?}", err),
    }
}

fn json<T>(o: &T) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection>
where
    T: ?Sized + Serialize,
{
    match serde_json::to_string(o) {
        Ok(json) => Ok(Box::new(json)),
        Err(err) => {
            let error = RPCError {
                error: err.to_string(),
            };
            let json = serde_json::to_string(&error).expect("Could not even serialize this error.");
            Ok(Box::new(warp::reply::with_status(
                json,
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}
