use crate::node::ArcState;
use crate::rpc::server::{RPCMessage, RpcMessageSender};
use crate::rpc::{ProcessRequest, ProcessResponse};
use crate::Result;
use tokio::sync::oneshot;

pub async fn handle_process(
    state: ArcState,
    tx: RpcMessageSender,
    request: ProcessRequest,
) -> Result<ProcessResponse> {
    // let (message, rx) = RPCMessage::new_with_channel(Target::BroadcastAllPeers);
    // tx.send(message).await.unwrap(); // TODO: unwrap

    todo!()
}
