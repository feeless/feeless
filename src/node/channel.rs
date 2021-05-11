use crate::network::Network;
use crate::node::controller::{Controller, Packet};
use crate::node::state::ArcState;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

/// A `channel` communicates with a peer over TCP. This will relay packets in and out of the
/// contained [Controller] which does all the work.
pub fn new_peer_channel(
    network: Network,
    state: ArcState,
    address: SocketAddr,
) -> anyhow::Result<()> {
    let (controller, tx, mut rx) = Controller::new_with_channels(network, state.clone(), address);

    tokio::spawn(controller.run());
    tokio::spawn(async move {
        let stream = TcpStream::connect(address)
            .await
            .expect(&format!("Failed to connect to {}", address));
        let (mut in_stream, mut out_stream) = stream.into_split();

        // Handle reads in a separate task.
        tokio::spawn(async move {
            let mut buffer: [u8; 10240] = [0; 10240];
            loop {
                let bytes = in_stream
                    .read(&mut buffer)
                    .await
                    .expect("Could not read from peer");

                tx.send(Packet::new(Vec::from(&buffer[0..bytes])))
                    .await
                    .expect("Could not send to controller");
            }
        });

        // Writing to the socket. Keep it in this task.
        loop {
            let to_send = match rx.recv().await {
                Some(bytes) => bytes,
                None => {
                    debug!("Could not recv packet for sending");
                    return;
                }
            };

            out_stream
                .write_all(&to_send.data)
                .await
                .expect("Could not send to socket");
        }
    });

    Ok(())
}
