mod channel;
mod controller;
mod cookie;
mod header;
mod messages;
mod peer;
mod state;
mod timestamp;
mod wire;

use crate::network::Network;
use crate::node::state::ArcState;
use anyhow::Context;
use channel::network_channel;
pub use controller::{Controller, Packet};
pub use header::Header;
pub use state::{MemoryState, SledDiskState};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
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

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut handles = vec![];
        let initial_peers = self.state.lock().await.peers().await?;
        for socket_addr in initial_peers {
            info!("Spawning a channel to {:?}", socket_addr);
            let state = self.state.clone();
            let network = self.network.clone();
            let handle = tokio::spawn(async move {
                let stream = TcpStream::connect(socket_addr).await.unwrap();
                network_channel(network, state, stream)
                    .await
                    .expect("Error in network_channel")
            });
            handles.push(handle)
        }

        for handle in handles {
            handle.await?
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
