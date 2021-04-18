//! Channel commands for controllers, i.e. messages to each peer handler.
use tokio::sync::{mpsc, oneshot};

pub type ControllerMessageSender = mpsc::Sender<ControllerCommand>;
pub type ControllerMessageReceiver = mpsc::Receiver<ControllerCommand>;

#[derive(Debug)]
pub enum ControllerCommand {
    // TODO: Broadcast(StateBlock),
    /// Requests information about a peer.
    PeerInfo(oneshot::Sender<crate::rpc::calls::DetailedPeerInfo>),
}
