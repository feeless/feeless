use crate::blocks::StateBlock;
use tracing::trace;

enum BootstrapState {
    Idle,
    FrontierStream,
    BulkPull,
}

pub struct BootstrapClient {
    state: BootstrapState,
}

impl BootstrapClient {
    pub async fn handle_bootstrap_state_block(state_block: &StateBlock) -> anyhow::Result<()> {
        trace!("Got bootstrap state block {}", state_block);
        Ok(())
    }

    // #[instrument(skip(self, packet))]
    // async fn handle_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
    //     trace!("handle_packet");
    //     debug_assert!(!packet.data.is_empty());
    //
    //     macro_rules! handle {
    //         ($self: ident, $fun:ident, $header:expr) => {{
    //             let sh = Some(&$header);
    //             let payload = self
    //                 .recv(sh)
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
    //     // TODO: Handle frontier stream
    //     // if self.frontier_stream {
    //     //     let payload = self.recv::<FrontierResp>(None).await?;
    //     //     self.handle_frontier_resp(payload).await?;
    //     // } else {
    //
    //     if matches!(self.bootstrap_state, BootstrapState::BulkPull) {
    //         let block_type_byte = self.incoming_buffer[0].to_owned();
    //         self.incoming_buffer = Vec::from(&self.incoming_buffer[1..]);
    //         let block_type = BlockType::try_from(block_type_byte)?;
    //         match block_type {
    //             BlockType::Invalid => {
    //                 error!("invalid block, not implemented")
    //             }
    //             BlockType::NotABlock => {
    //                 trace!("Received NotABlock, reverting to Idle bootstrap state");
    //                 self.bootstrap_state = Idle
    //             }
    //             BlockType::Send => {
    //                 info!("send block, not implemented");
    //                 self.incoming_buffer = Vec::from(&self.incoming_buffer[SendBlock::LEN..])
    //             }
    //             BlockType::Receive => {
    //                 info!("receive block, not implemented");
    //                 self.incoming_buffer = Vec::from(&self.incoming_buffer[ReceiveBlock::LEN..])
    //             }
    //             BlockType::Open => {
    //                 info!("open block, not implemented");
    //                 self.incoming_buffer = Vec::from(&self.incoming_buffer[OpenBlock::LEN..])
    //             }
    //             BlockType::Change => {
    //                 info!("change block, not implemented");
    //                 self.incoming_buffer = Vec::from(&self.incoming_buffer[ChangeBlock::LEN..])
    //             }
    //             BlockType::State => {
    //                 let payload: Option<StateBlock> = self.recv(None)?;
    //                 if let Some(payload) = payload {
    //                     match &self.last_annotation {
    //                         Some(a) => info!("{} {:?}", a, &payload),
    //                         None => debug!("{:?}", &payload),
    //                     };
    //
    //                     BootstrapClient::handle_bootstrap_state_block(&payload).await?;
    //                 } else {
    //                     warn!("Expected payload not received")
    //                 }
    //             }
    //         }
    //     }
    //
    //     Ok(())
    // }
}
