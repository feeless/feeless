use std::collections::VecDeque;

use crate::bootstrap::messages::bulk_pull::BulkPull;
use crate::bootstrap::server::ParserState::ReceivingPayload;
use crate::node::{Packet, Wire};
use crate::transport::header::{Header, MessageType};
use crate::transport::RecvResult::NotEnoughBytes;
use crate::transport::RecvResult::Received;
use crate::transport::{recv, recv_header, recv_payload, RecvResult};
use crate::Network;
use anyhow::anyhow;
use anyhow::Context;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use tracing::{instrument, trace, warn};

enum ParserState {
    ReceivingHeader,
    ReceivingPayload(Header),
}

pub struct BootstrapServer {
    parser_state: ParserState,
    incoming_buffer: VecDeque<u8>,
}

// CONCLUSIONI
/*
Tenere tutto com'e' per ora, esternalizzare la macro, tenere il parsing di header e payload insieme,
quindi riutilizzare tutto il codice che e' stato gia' scritto. Unica cosa usare VecDeque invece
che Vec e usare drain invece che le due righe.
Ultima cosa da provare, vedere se invece di usare un loop con una macchina a stati si puo' fare
iterativo dato che dovrebbe avere solo pochi stati (ma questo si puo' fare anche dopo aver portato
o creato qualche test, in modo da vedere se funziona).
 */

// macro_rules! handle2 {
//     ($self: ident, $fun: ident, $mut_buffer: expr, $header: ident) => {{
//         match crate::transport::recv_payload(&$header, &mut $mut_buffer)? {
//             Received(message) => {
//                 $self.$fun(&$header, message).await?;
//                 Ok(ParserState::ReceivingHeader)
//             }
//             NotEnoughBytes => Ok(ParserState::ReceivingPayload($header)),
//         }
//     };};
// }
struct FrontierReq {}

impl Debug for FrontierReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Wire for FrontierReq {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(header: Option<&Header>, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn len(header: Option<&Header>) -> anyhow::Result<usize>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl BootstrapServer {
    pub(crate) fn new() -> Self {
        BootstrapServer {
            parser_state: ParserState::ReceivingHeader,
            incoming_buffer: VecDeque::new(),
        }
    }

    pub async fn handle_bulk_pull(
        &mut self,
        _header: Header,
        _bulk_pull: BulkPull,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn handle_frontier_req(
        &mut self,
        _header: Header,
        _frontier_req: FrontierReq,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle<'r, T: Wire + Debug, Fut>(
        &'r mut self,
        fun: fn(&'r mut BootstrapServer, Header, T) -> Fut,
        header: Header,
    ) -> anyhow::Result<ParserState>
    where
        Fut: Future<Output = anyhow::Result<()>> + 'r,
    {
        match recv_payload(&header, &mut self.incoming_buffer)? {
            Received(message) => {
                fun(self, header, message).await?;
                Ok(ParserState::ReceivingHeader)
            }
            NotEnoughBytes => Ok(ParserState::ReceivingPayload(header)),
        }
    }

    async fn handle_mess(&mut self, header: Header) -> anyhow::Result<ParserState> {
        let message_type = &header.message_type();
        match message_type {
            // MessageType::Keepalive => {}
            // MessageType::Publish => {}
            // MessageType::ConfirmReq => {}
            // MessageType::ConfirmAck => {}
            MessageType::BulkPull => self.handle(Self::handle_bulk_pull, header).await,
            // MessageType::BulkPush => {}
            MessageType::FrontierReq => self.handle(Self::handle_frontier_req, header).await,
            // MessageType::Handshake => {}
            // MessageType::BulkPullAccount => {}
            // MessageType::TelemetryReq => {}
            // MessageType::TelemetryAck => {}
            _ => Err(anyhow!("Unhandled message with header: {:?}", &header)),
        }
    }

    #[instrument(skip(self, packet))]
    async fn handle_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
        trace!("[bootstrap server] handle_packet");
        //
        //     if let Some(annotation) = packet.annotation {
        //         self.last_annotation = Some(annotation);
        //     }
        self.incoming_buffer.extend(packet.data);

        match self.parser_state {
            ParserState::ReceivingHeader => {
                match recv_header(&mut self.incoming_buffer)? {
                    Received(header) => self.parser_state = self.handle_mess(header).await?,
                    NotEnoughBytes => return Ok(()),
                }
                // match recv_header(&mut self.incoming_buffer)? {
                //     Received(header) => {
                //         header.validate(&Network::Live)?; // TODO: parametrize
                //                                           // we got the header next bytes are the payload
                //         self.parser_state = ReceivingPayload(header);
                //         // self.parse_payload
                //         match recv_payload(&header, &mut self.incoming_buffer) {}
                //     }
                //     RecvResult::NotEnoughBytes => {
                //         // let's wait for another packet
                //         return Ok(());
                //     }
                // }
            }
            ParserState::ReceivingPayload(header) => {}
        }

        Ok(())
        //
        //     loop {
        //         match &self.parser_state {
        //             ParserState::ReceivingHeader => {
        //                 // received n bytes, of which the first h are expected to be header
        //                 if self.incoming_buffer.len() < Header::LEN {
        //                     break; // EXIT this function and wait for a new packet
        //                 } else {
        //                     if let Some(header) = self.parse_header() {
        //                         header.validate(&Network::Live)?; // TODO: parametrize
        //                         self.parser_state = ReceivingPayload(header)
        //                     }
        //                 }
        //             }
        //             ParserState::ReceivingPayload(header) => {
        //                 // received n bytes, of which the first p are expected to be payload
        //                 trace!("Attempt to handle: {:?}", header.message_type());
        //                 match header.message_type() {
        //                     // MessageType::Keepalive => {}
        //                     // MessageType::Publish => {}
        //                     // MessageType::ConfirmReq => {}
        //                     // MessageType::ConfirmAck => {}
        //                     // MessageType::FrontierReq => {}
        //                     // MessageType::Handshake => {}
        //                     // MessageType::TelemetryReq => {}
        //                     // MessageType::TelemetryAck => {}
        //                     MessageType::BulkPull => {
        //                         let bulk_pull = self.parse_message(header)?;
        //                         self.handle_bulk_pull(header, bulk_pull)
        //                     }
        //                     // MessageType::BulkPush => {}
        //                     // MessageType::BulkPullAccount => {}
        //                     _ => return Err(anyhow!("Unhandled message with header: {:?}", header)),
        //                 }
        //             }
        //         };
        //         // TODO: state transition happens here
        //     }
        //
        //     loop {
        //         let (new_state, keep_processing) = match self.recv_state {
        //             RecvState::Header => {
        //                 if let Some(header) = self.recv::<Header>(None)? {
        //                     header.validate(&self.network)?;
        //                     (RecvState::Payload(header), true)
        //                 } else {
        //                     (RecvState::Header, false)
        //                 }
        //             }
        //             RecvState::Payload(header) => {
        //                 trace!(
        //                     "Attempt to handle message of type: {:?}",
        //                     header.message_type()
        //                 );
        //                 match header.message_type() {
        //                     // MessageType::Keepalive => handle!(self, handle_keepalive, header),
        //                     // MessageType::Publish => handle!(self, handle_publish, header),
        //                     // MessageType::ConfirmReq => handle!(self, handle_confirm_req, header),
        //                     // MessageType::ConfirmAck => handle!(self, handle_confirm_ack, header),
        //                     // MessageType::FrontierReq => handle!(self, handle_frontier_req, header),
        //                     // MessageType::Handshake => handle!(self, handle_handshake, header),
        //                     // MessageType::TelemetryReq => {
        //                     //     handle!(self, handle_telemetry_req, header)
        //                     // }
        //                     // MessageType::TelemetryAck => {
        //                     //     handle!(self, handle_telemetry_ack, header)
        //                     // }
        //                     MessageType::BulkPull => handle!(self, handle_bulk_pull, header),
        //                     // MessageType::BulkPush => {}
        //                     // MessageType::BulkPullAccount => {}
        //                     _ => return Err(anyhow!("Unhandled message: {:?}", header)),
        //                 };
        //                 (RecvState::Header, false)
        //             }
        //         };
        //         self.recv_state = new_state;
        //         if !keep_processing {
        //             break;
        //         }
        //     }
        //     Ok(())
        // }
    }

    // async fn parse_message_whole<T: Wire + Debug>(
    //     &mut self,
    //     header: Header,
    // ) -> anyhow::Result<ParserState> {
    //     let message_type = header.message_type();
    //     let incoming_buffer = &mut self.incoming_buffer;
    //     match message_type {
    //         // MessageType::Keepalive => {}
    //         // MessageType::Publish => {}
    //         // MessageType::ConfirmReq => {}
    //         // MessageType::ConfirmAck => {}
    //         MessageType::BulkPull => Ok(handle!(self, handle_bulk_pull, header, incoming_buffer)),
    //         // MessageType::BulkPush => {}
    //         // MessageType::FrontierReq => {}
    //         // MessageType::Handshake => {}
    //         // MessageType::BulkPullAccount => {}
    //         // MessageType::TelemetryReq => {}
    //         // MessageType::TelemetryAck => {}
    //         _ => Err(anyhow!("Unhandled message: {:?}", header)),
    //     }
    // }
}
