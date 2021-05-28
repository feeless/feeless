use crate::network::Network;
use crate::node::peer::{Packet, Peer};
use crate::node::state::ArcState;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

/// A `channel` communicates with a peer over TCP. This will relay packets in and out of the
/// contained [Peer] which does all the work.
pub fn new_peer_channel(
    network: Network,
    state: ArcState,
    address: SocketAddr,
) -> anyhow::Result<()> {
    let (peer, tx, mut rx) = Peer::new_with_channels(network, state.clone(), address);

    tokio::spawn(peer.run());
    tokio::spawn(async move {
        let stream = TcpStream::connect(address)
            .await
            .expect(&format!("Failed to connect to {}", address));
        let (mut tcp_in, mut tcp_out) = stream.into_split();

        // Handle reads in a separate task.
        tokio::spawn(async move {
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

        // Writing to the socket. Keep it in this task.
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

    Ok(())
}
