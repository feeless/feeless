use std::collections::VecDeque;

use crate::bootstrap::messages::bulk_pull::BulkPull;
use crate::transport::header::Header;

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
impl BootstrapServer {
    pub(crate) fn new() -> Self {
        BootstrapServer {
            parser_state: ParserState::ReceivingHeader,
            incoming_buffer: VecDeque::new(),
        }
    }

    pub async fn handle_bulk_pull(
        &mut self,
        _header: &Header,
        _bulk_pull: BulkPull,
    ) -> anyhow::Result<()> {
        // if matches!(self.bootstrap_state, FrontierStream) {
        //     panic!("Invalid bootstrap state transition FrontierStream => BulkPull");
        // }
        Ok(())
    }

    // /// Receive from the incoming buffer for type `Header`. Will return None if there aren't enough
    // /// bytes available.
    // #[instrument(skip(self))]
    // fn parse_header(&mut self) -> Option<Header> {
    //     let header_bytes = self
    //         .incoming_buffer
    //         .drain(..Header::LEN)
    //         .collect::<Vec<u8>>();
    //     trace!("Header HEX: {}", to_hex(&header_bytes));
    //     Header::try_from(header_bytes).ok()
    // }
    //
    // /// Receive from the incoming buffer for type `T`. Will return None if there aren't enough
    // /// bytes available.
    // #[instrument(skip(self, header))]
    // fn parse_message<T: Wire + Debug>(&mut self, header: &Header) -> Option<T> {
    //     let message_byte_length = T::len_payload(header)?;
    //     if self.incoming_buffer.len() < message_byte_length {
    //         trace!(
    //             "Not enough bytes. Got {}, expected {}.",
    //             self.incoming_buffer.len(),
    //             message_byte_length
    //         );
    //         return None;
    //     }
    //
    //     let t_bytes = self
    //         .incoming_buffer
    //         .drain(..message_byte_length)
    //         .collect::<Vec<u8>>();
    //     trace("Message HEX: {}", to_hex(&t_bytes));
    //     T::deserialize_payload(header, &t_bytes).ok()
    // }
    //
    // #[instrument(skip(self, packet))]
    // async fn handle_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
    //     trace!("handle_packet");
    //
    //     macro_rules! handle_message {
    //         ($self: ident, $fun:ident, $header:ident) => {{
    //             let payload = self
    //                 .parse_message(header)
    //                 .with_context(|| format!("Receiving payload for {:?}", $header))?;
    //
    //             if let Some(payload) = payload {
    //                 match &self.last_annotation {
    //                     Some(a) => info!("{} {:?}", a, &payload),
    //                     None => debug!("{:?}", &payload),
    //                 };
    //
    //                 $self
    //                     .$fun(&$header, payload)
    //                     .await
    //                     .with_context(|| format!("Handling payload for {:?}", $header))?;
    //             } else {
    //             }
    //         };};
    //     }
    //
    //     if let Some(annotation) = packet.annotation {
    //         self.last_annotation = Some(annotation);
    //     }
    //     self.incoming_buffer.extend(packet.data);
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
