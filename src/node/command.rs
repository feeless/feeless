//! Channel commands for a node. Messages can be sent from the RPC server.
use tokio::sync::{mpsc, oneshot};

pub type NodeCommandSender = mpsc::Sender<NodeCommand>;
pub type NodeCommandReceiver = mpsc::Receiver<NodeCommand>;

pub type PeerInfoResponseSender = oneshot::Sender<crate::rpc::calls::Peers>;

#[derive(Debug)]
pub enum NodeCommand {
    /// Request all currently connected peers.
    PeerInfo(PeerInfoResponseSender),
}
