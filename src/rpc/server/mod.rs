use crate::node::ArcState;
use crate::rpc::client::RPCError;
use crate::rpc::Command;
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::{info, trace};
use warp::http::StatusCode;
use warp::Filter;

pub struct RPCMessage {
    response_rx: (),
}

pub struct RPCServer {
    state: ArcState,
    tx: mpsc::Sender<RPCMessage>,
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
            .and(warp::body::json())
            .and_then(Self::handle);

        warp::serve(rpc).run(([127, 0, 0, 1], 7076)).await;
        Ok(())
    }

    async fn handle(cmd: Command) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
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
            action => json(&RPCError {
                error: format!("The action: {:?} is unhandled", action),
            }),
        }
    }
}

fn json<T>(o: &T) -> Result<Box<dyn warp::Reply>, warp::Rejection>
where
    T: ?Sized + Serialize,
{
    let result = serde_json::to_string(o);
    match result {
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
