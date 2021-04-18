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
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info};
pub use wire::Wire;

pub struct Node {
    network: Network,
    state: ArcState,
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

    pub async fn run(self, mut node_rx: NodeCommandReceiver) -> anyhow::Result<()> {
        let mut controller_cmd_txs = vec![];
        let initial_peers = self.state.lock().await.peers().await?;
        for address in initial_peers {
            info!("Spawning a channel to {:?}", address);
            let state = self.state.clone();
            let network = self.network.clone();
            let controller_cmd_tx = new_peer_channel(network, state, address)?;
            controller_cmd_txs.push(controller_cmd_tx);
        }

        while let Some(node_command) = node_rx.recv().await {
            dbg!("todo node command", &node_command);
            match node_command {
                // TODO: broadcast to all controllers
                NodeCommand::PeerInfo(tx) => tx.send(crate::rpc::calls::Peers::Simple(vec![])),
            };
            // for node_rpc_tx in &controller_cmd_txs {
            //     node_rpc_tx.send(node_command.clone()).await;
            // }
        }

        info!("Quitting...");
        Ok(())
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
