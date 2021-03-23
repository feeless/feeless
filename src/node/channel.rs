use crate::network::Network;
use crate::node::controller::{Controller, Packet};
use crate::node::state::ArcState;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn network_channel(
    network: Network,
    state: ArcState,
    stream: TcpStream,
) -> anyhow::Result<()> {
    // TODO: How would this fail?
    let peer_addr = stream.peer_addr().unwrap();

    let (controller, tx, mut rx) = Controller::new_with_channels(network, state, peer_addr);

    // We don't `await` here since the controller will quit when the incoming channel drops.
    tokio::spawn(controller.run());

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
            None => return Ok(()),
        };

        out_stream.write_all(&to_send.data).await?;
    }
}
