mod channel;
mod command;
mod cookie;
mod header;
mod messages;
mod peer;
mod peer_info;
mod state;
mod timestamp;
mod wire;

use crate::rpc::server::RPCServer;
use crate::Network;
pub use crate::Version;
use anyhow::Context;
pub use command::{NodeCommand, NodeCommandReceiver, NodeCommandSender};
pub use header::Header;
pub use peer::{Packet, Peer};
pub use state::{ArcState, MemoryState, SledDiskState};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, info};
pub use wire::Wire;

pub struct Node {
    network: Network,
    state: ArcState,
}

impl Node {
    pub async fn start(override_peers: Option<Vec<String>>) -> anyhow::Result<()> {
        let mut node = Node::new(Network::Live);
        let rpc_rx = node.start_rpc_server().await?;
        if let Some(str_addrs) = override_peers {
            let mut socket_addrs = vec![];
            for str_addr in str_addrs {
                let socket_addr = SocketAddr::from_str(&str_addr)
                    .with_context(|| format!("Could not parse host:port: {}", str_addr))?;
                socket_addrs.push(socket_addr);
            }
            node.add_peers(&socket_addrs).await?;
        } else {
            node.peer_autodiscovery().await?;
        }

        node.run(rpc_rx).await
    }

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
        let initial_peers = self.state.lock().await.peers().await?;
        for address in initial_peers {
            info!("Spawning a connection to {:?}", address);
            let state = self.state.clone();
            let network = self.network.clone();
            Self::spawn_tcp_peer(network, state, address).await?;
        }

        while let Some(node_command) = node_rx.recv().await {
            dbg!("todo node command", &node_command);
            match node_command {
                NodeCommand::PeerInfo(_tx) => todo!("get_active_peers()"),
            };
        }

        info!("Quitting...");
        Ok(())
    }

    pub async fn spawn_tcp_peer(
        network: Network,
        state: ArcState,
        address: SocketAddr,
    ) -> anyhow::Result<()> {
        let (peer, tx, mut rx) = Peer::new_with_channels(network, state.clone(), address);

        // Task for the Peer handler.
        let peer_task = tokio::spawn(peer.run());

        let stream = TcpStream::connect(address)
            .await
            .expect(&format!("Failed to connect to {}", address));
        let (mut tcp_in, mut tcp_out) = stream.into_split();

        // Handle reads in a separate task.
        let reader_task = tokio::spawn(async move {
            let mut buffer: [u8; 10240] = [0; 10240];
            loop {
                let bytes = tcp_in
                    .read(&mut buffer)
                    .await
                    .expect("Could not read from socket");

                let result = tx.send(Packet::new(Vec::from(&buffer[0..bytes]))).await;
                if result.is_err() {
                    // When the channel disconnects from Peer, we rely on Peer to report the error.
                    break;
                }
            }
        });

        // Handle writes in a separate task.
        let writer_task = tokio::spawn(async move {
            loop {
                let to_send = match rx.recv().await {
                    Some(bytes) => bytes,
                    None => {
                        // When the channel disconnects from Peer, we rely on Peer to report the error.
                        return;
                    }
                };

                tcp_out
                    .write_all(&to_send.data)
                    .await
                    .expect("Could not send to socket");
            }
        });

        tokio::join!(node)

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
