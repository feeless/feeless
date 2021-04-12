use crate::node::ArcState;
use crate::rpc::server::{RPCMessage, RpcMessageSender, Target};
use crate::rpc::{ProcessRequest, ProcessResponse};
use crate::Result;
use tokio::sync::oneshot;

pub async fn handle_process(
    state: ArcState,
    tx: RpcMessageSender,
    request: ProcessRequest,
) -> Result<ProcessResponse> {
    let (message, rx) = RPCMessage::new_with_channel(Target::BroadcastAllPeers).await;
    tx.send(message).await;

    todo!()
}
