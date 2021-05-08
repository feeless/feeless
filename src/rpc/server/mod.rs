use crate::node::{ArcState, NodeCommandReceiver, NodeCommandSender};
use crate::rpc::client::RPCError;
use crate::rpc::{NodeHandler, RpcCommand};
use crate::Result;
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::{info, trace};
use warp::http::StatusCode;
use warp::Filter;

// pub trait NodeHandler {
//     fn handle();
// }

// pub type RpcMessageSender = mpsc::Sender<RPCMessage>;
// pub type RpcMessageReceiver = mpsc::Receiver<RPCMessage>;
// pub type RpcMessageResponseSender = oneshot::Sender<CommandResponse>;
// pub type RpcMessageResponseReceiver = oneshot::Receiver<CommandResponse>;
//
// #[derive(Debug, Clone)]
// pub struct RPCMessage {
//     command: RpcCommand,
//     tx: RpcMessageResponseSender,
// }
//
// impl RPCMessage {
//     fn new_with_channel(command: RpcCommand) -> (Self, RpcMessageResponseReceiver) {
//         let (tx, rx) = oneshot::channel();
//         (Self { command, tx }, rx)
//     }
// }

pub struct RPCServer {
    state: ArcState,
    node_cmd_tx: NodeCommandSender,
}

impl RPCServer {
    pub fn new_with_channel(state: ArcState) -> (Self, NodeCommandReceiver) {
        let (tx, rx) = mpsc::channel(100);
        let s = Self {
            node_cmd_tx: tx,
            state,
        };
        (s, rx)
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!("Starting RPC server");
        let rpc = warp::post()
            .and(warp::body::content_length_limit(1024 * 16))
            .and(with_state(self.state.clone()))
            .and(with_node_tx(self.node_cmd_tx.clone()))
            .and(warp::body::json())
            .and_then(Self::handle);

        // TODO: Configurable
        warp::serve(rpc).run(([127, 0, 0, 1], 7076)).await;
        Ok(())
    }

    async fn handle(
        _state: ArcState,
        node_tx: NodeCommandSender,
        cmd: RpcCommand,
    ) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
        trace!("Handling command: {:?}", cmd);
        match &cmd {
            // TODO: Example usage
            // Command::ActiveDifficulty(c) => json(&ActiveDifficultyResponse {
            //     multiplier: 1.5,
            //     network_current: Difficulty::new(1),
            //     network_minimum: Difficulty::new(2),
            //     network_receive_current: Difficulty::new(3),
            //     network_receive_minimum: Difficulty::new(4),
            // }),
            // RpcCommand::Peers(c) => json_result(handle_peers(state, tx, c).await),
            RpcCommand::Peers(c) => json_result(c.handle(node_tx).await),
            // RpcCommand::Process(c) => json_result(handle_process(state, tx, c).await),
            action => json_result(Ok(RPCError {
                error: format!("This action is unhandled by the RPC server: {:?}", action),
            })),
        }
    }
}

fn with_node_tx(
    node_cmd_tx: NodeCommandSender,
) -> impl Filter<Extract = (NodeCommandSender,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || node_cmd_tx.clone())
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
