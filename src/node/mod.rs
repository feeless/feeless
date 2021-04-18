mod channel;
mod command;
mod controller;
mod cookie;
mod header;
mod messages;
mod peer;
mod state;
mod timestamp;
mod wire;

use crate::node::command::PeerInfoResponseSender;
use crate::node::controller::{ControllerCommand, ControllerMessageSender};
use crate::rpc::server::RPCServer;
use crate::Network;
pub use crate::Version;
use anyhow::Context;
use channel::new_peer_channel;
pub use command::{NodeCommand, NodeCommandReceiver, NodeCommandSender};
pub use controller::{Controller, Packet};
pub use header::Header;
pub use state::{ArcState, MemoryState, SledDiskState};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, info};
pub use wire::Wire;

pub struct Node {
    network: Network,
    state: ArcState,
    controller_cmd_txs: Vec<ControllerMessageSender>,
}

impl Node {
    pub fn new(network: Network) -> Self {
        // let state = SledDiskState::new(Network::Live);
        let state = MemoryState::new(network);
        let state = Arc::new(Mutex::new(state));
        Self { state, network }
    }

    pub async fn start_rpc_server(&self) -> anyhow::Result<NodeCommandReceiver> {
        let (rpc_server, rx) = RPCServer::new_with_channel(self.state.clone());
        tokio::spawn(rpc_server.run());
        Ok(rx)
    }

    pub async fn run(mut self, mut node_rx: NodeCommandReceiver) -> anyhow::Result<()> {
        let initial_peers = self.state.lock().await.peers().await?;
        for address in initial_peers {
            info!("Spawning a channel to {:?}", address);
            let state = self.state.clone();
            let network = self.network.clone();
            let controller_cmd_tx = new_peer_channel(network, state, address)?;
            self.controller_cmd_txs.push(controller_cmd_tx);
        }

        // TODO: The node might need to receive commands from the controllers, e.g. found knowledge
        //       of new peers, to decide if it should connect to these new nodes. <-- goodish
        //       That or if the controller spawns another controller, the node will need to access
        //       the controller message channel. <-- probably not a good solution
        //       Another idea: Just have the node poll every few seconds to see if there are any
        //       new peers in its state (which is added by the controller), then connect
        //       to them appropriately. <-- I like this the best! It means no need for a channel back.
        while let Some(node_command) = node_rx.recv().await {
            dbg!("todo node command", &node_command);
            match node_command {
                // TODO: broadcast to all controllers
                NodeCommand::PeerInfo(tx) => self.request_peer_info(tx),
                // NodeCommand::PeerInfo(tx) => tx.send(crate::rpc::calls::Peers::Simple(vec![])),
            };
            // for node_rpc_tx in &Node {
            //     node_rpc_tx.send(node_command.clone()).await;
            // }
        }

        info!("Quitting...");
        Ok(())
    }

    async fn request_peer_info(&self, response: PeerInfoResponseSender) {
        let (tx, rx) = mpsc::channel(100);
        for controller_tx in self.controller_cmd_txs {
            tokio::spawn(async move {
                let (controller_tx, controller_rx) = oneshot::channel();
                tx.send(ControllerCommand::PeerInfo(controller_tx))
                    .await
                    .expect("TODO");
            });
        }
    }

    pub async fn add_peers(&mut self, socket_addrs: &[SocketAddr]) -> anyhow::Result<()> {
        debug!("Adding peers to state: {:?}", socket_addrs);
        self.state.lock().await.add_peers(socket_addrs).await?;
        Ok(())
    }

    pub async fn peer_autodiscovery(&mut self) -> anyhow::Result<()> {
        let host = self.network.peering_host();
        info!("Peer autodiscovery initiated with {}", host);
        let socket_addrs: Vec<SocketAddr> = tokio::net::lookup_host(host)
            .await
            .context("Error while trying to lookup default peers")?
            .collect();
        self.add_peers(&socket_addrs).await?;
        Ok(())
    }
}
