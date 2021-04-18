//! Channel commands for a node. Messages can be sent from the RPC server.
use tokio::sync::{mpsc, oneshot};

pub type NodeCommandSender = mpsc::Sender<NodeCommand>;
pub type NodeCommandReceiver = mpsc::Receiver<NodeCommand>;

#[derive(Debug)]
pub enum NodeCommand {
    /// Request all currently connected peers.
    PeerInfo(oneshot::Sender<crate::rpc::calls::Peers>),
}
