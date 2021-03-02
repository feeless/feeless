use crate::node::cookie::Cookie;
use crate::node::header::{Extensions, Header, MessageType};
use crate::node::messages::confirm_ack::ConfirmAck;
use crate::node::messages::confirm_req::ConfirmReq;
use crate::node::messages::handshake::{Handshake, HandshakeQuery, HandshakeResponse};
use crate::node::messages::keepalive::Keepalive;
use crate::node::messages::publish::Publish;
use crate::node::messages::telemetry_ack::TelemetryAck;

use crate::network::Network;
use crate::node::controller::Controller;
use crate::node::state::{ArcState, DynState};
use crate::node::wire::Wire;
use crate::{expect_len, to_hex, Public, Seed, Signature};
use anyhow::{anyhow, Context};
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tracing::{debug, instrument, trace, warn};

// /// A connection to a single peer.
// pub struct Channel {
//     network: Network,
//
//     pub state: ArcState,
//
//     // TODO: Both of these into a Communication trait, for ease of testing. e.g.:
//     //  * async fn Comm::send() -> Result<()>
//     //  * async fn Comm::recv() -> Result<()>
//     //  * fn Comm::address() -> String
//     //
//     // This would also remove Self::buffer.
//     // Not sure about the performance problems of having to use async-trait.
//     stream: TcpStream,
//     pub peer_addr: SocketAddr,
//
//     /// A reusable header to reduce allocations.
//     pub header: Header,
//
//     /// Storage that can be shared within this task without reallocating.
//     /// This is currently only used for the recv buffers.
//     buffer: Vec<u8>,
// }

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

            tx.send(Vec::from(&buffer[0..bytes]))
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

        out_stream.write_all(&to_send).await?;
    }
}

//     impl Channel {
//         pub async fn new(network: Network, state: ArcState, stream: TcpStream) -> Self {
//         // TODO: Remove unwrap?
//         let peer_addr = stream.peer_addr().unwrap();
//
//         Self {
//             network,
//             state,
//             stream,
//             peer_addr,
//             header: Header::new(network, MessageType::Handshake, Extensions::new()),
//             buffer: Vec::with_capacity(1024),
//         }
//     }
//
//
//         todo!()
//
//
//         // trace!("Loop start");
//         // loop {
//         //     let header = self
//         //         .recv::<Header>(None)
//         //         .await
//         //         .with_context(|| format!("Main node loop"))?;
//         //     header.validate(&self.controller.network())?;
//         //     trace!("Header: {:?}", &header);
//         //
//         //     match header.message_type() {
//         //         MessageType::Keepalive => self.recv_keepalive(header).await?,
//         //         MessageType::Publish => self.recv_publish(header).await?,
//         //         MessageType::ConfirmReq => self.recv_confirm_req(header).await?,
//         //         MessageType::ConfirmAck => self.recv_confirm_ack(header).await?,
//         //         // MessageType::BulkPull => todo!(),
//         //         // MessageType::BulkPush => todo!(),
//         //         // MessageType::FrontierReq => todo!(),
//         //         MessageType::Handshake => self.recv_node_id_handshake(header).await?,
//         //         // MessageType::BulkPullAccount => todo!(),
//         //         MessageType::TelemetryReq => self.recv_telemetry_req(header).await?,
//         //         MessageType::TelemetryAck => self.recv_telemetry_ack(header).await?,
//         //         _ => todo!("{:?}", header),
//         //     }
//         // }
//     //
//     // #[instrument(skip(self))]
//     // async fn recv_keepalive(&mut self, header: Header) -> anyhow::Result<()> {
//     //     let keepalive = self.recv::<Keepalive>(Some(&header)).await?;
//     //     debug!("{:?}", keepalive);
//     //     Ok(())
//     // }
//     //
//     // #[instrument(skip(self, header))]
//     // async fn recv_publish(&mut self, header: Header) -> anyhow::Result<()> {
//     //     let publish = self.recv::<Publish>(Some(&header)).await?;
//     //     dbg!(publish);
//     //     // todo!();
//     //     Ok(())
//     // }
//     //
//     // #[instrument(skip(self, header))]
//     // async fn recv_confirm_req(&mut self, header: Header) -> anyhow::Result<()> {
//     //     let data = self.recv::<ConfirmReq>(Some(&header)).await?;
//     //     trace!("TODO confirm req pairs: {:?}", &data);
//     //     // warn!("TODO confirm_req");
//     //     Ok(())
//     // }
//     //
//     // #[instrument(skip(self, header))]
//     // async fn recv_confirm_ack(&mut self, header: Header) -> anyhow::Result<()> {
//     //     let vote = self.recv::<ConfirmAck>(Some(&header)).await?;
//     //
//     //     dbg!(&vote);
//     //     self.controller.add_vote(&vote).await?;
//     //
//     //     Ok(())
//     // }
//     //
//     // #[instrument(skip(self))]
//     // async fn recv_telemetry_req(&mut self, header: Header) -> anyhow::Result<()> {
//     //     // Nothing else to receive!
//     //     Ok(())
//     // }
//     //
//     // #[instrument(skip(self))]
//     // async fn send_telemetry_req(&mut self) -> anyhow::Result<()> {
//     //     self.send_header(MessageType::TelemetryReq, Extensions::new())
//     //         .await?;
//     //     Ok(())
//     // }
//     //
//     // #[instrument(skip(self))]
//     // async fn recv_telemetry_ack(&mut self, header: Header) -> anyhow::Result<()> {
//     //     let telemetry = self.recv::<TelemetryAck>(Some(&header)).await?;
//     //     dbg!(telemetry);
//     //     Ok(())
//     // }
// }
