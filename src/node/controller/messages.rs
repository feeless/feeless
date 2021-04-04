use super::Controller;
use crate::node::cookie::Cookie;
use crate::node::header::{Extensions, Header, MessageType};
use crate::node::messages::confirm_ack::ConfirmAck;
use crate::node::messages::confirm_req::ConfirmReq;
use crate::node::messages::frontier_req::FrontierReq;
use crate::node::messages::frontier_resp::FrontierResp;
use crate::node::messages::handshake::{Handshake, HandshakeQuery, HandshakeResponse};
use crate::node::messages::keepalive::Keepalive;
use crate::node::messages::publish::Publish;
use crate::node::messages::telemetry_ack::TelemetryAck;
use crate::node::messages::telemetry_req::TelemetryReq;
use crate::{Public, Seed, Signature};
use anyhow::Context;
use tracing::{debug, instrument, trace, warn};
use crate::blocks::{BlockHolder, Block, BlockHash};

impl Controller {
    #[instrument(skip(self))]
    pub async fn send_handshake(&mut self) -> anyhow::Result<()> {
        trace!("Sending handshake");
        self.send_header(MessageType::Handshake, *Extensions::new().query())
            .await?;

        // TODO: Track our own cookie?
        let cookie = Cookie::random();
        self.state
            .lock()
            .await
            .set_cookie(self.peer_addr, cookie.clone())
            .await?;
        let handshake_query = HandshakeQuery::new(cookie);
        self.send(&handshake_query).await?;

        Ok(())
    }

    pub async fn handle_handshake(
        &mut self,
        header: &Header,
        handshake: Handshake,
    ) -> anyhow::Result<()> {
        enum ShouldRespond {
            No,
            Yes(Public, Signature),
        }
        let mut should_respond = ShouldRespond::No;

        if header.ext().is_query() {
            // This would probably be a programming error if it panicked.
            let query = handshake.query.expect("query is None but is_query is True");

            // XXX: Hacky code here just to see if it works!
            // TODO: Move into state
            let seed = Seed::random();
            let private = seed.derive(0);
            let public = private.to_public()?;
            let signature = private.sign(query.cookie().as_bytes())?;
            public
                .verify(query.cookie().as_bytes(), &signature)
                .context("Verify recv handshake signature")?;

            // Respond at the end because we mess with the header buffer.
            should_respond = ShouldRespond::Yes(public, signature);
        }

        if header.ext().is_response() {
            let response = handshake
                .response
                .expect("response is None but is_response is True");
            let public = response.public;
            let signature = response.signature;

            // TODO: Move to controller
            let cookie = &self
                .state
                .lock()
                .await
                .cookie_for_socket_addr(&self.peer_addr)
                .await?;
            if cookie.is_none() {
                warn!(
                    "Peer {:?} has no cookie. Can't verify handshake.",
                    self.peer_addr
                );
                return Ok(());
            }
            let cookie = cookie.as_ref().unwrap();

            if self.validate_handshakes {
                public
                    .verify(&cookie.as_bytes(), &signature)
                    .context("Invalid signature in handshake response")?;
            }
        }

        if let ShouldRespond::Yes(public, signature) = should_respond {
            let mut header = self.header;
            header.reset(MessageType::Handshake, *Extensions::new().response());
            self.send(&header).await?;

            let response = HandshakeResponse::new(public, signature);
            self.send(&response).await?;
        }

        Ok(())
    }

    pub async fn handle_keepalive(
        &mut self,
        _header: &Header,
        keepalive: Keepalive,
    ) -> anyhow::Result<()> {
        // dbg!(keepalive);
        debug!("{:?}", keepalive);
        Ok(())
    }

    pub async fn handle_telemetry_req(
        &mut self,
        _header: &Header,
        _telemetry_req: TelemetryReq,
    ) -> anyhow::Result<()> {
        // dbg!(telemetry_req);
        Ok(())
    }

    pub async fn handle_telemetry_ack(
        &mut self,
        _header: &Header,
        _telemetry_ack: TelemetryAck,
    ) -> anyhow::Result<()> {
        // dbg!(telemetry_ack);
        Ok(())
    }

    pub async fn handle_publish(
        &mut self,
        _header: &Header,
        publish: Publish,
    ) -> anyhow::Result<()> {
        // dbg!(publish);
        let _block = match publish.block_holder {
            BlockHolder::Send(_) => {
                todo!("Received a send block")
            }
            BlockHolder::Receive(_) => {
                todo!("Received a receive block")
            }
            BlockHolder::Open(_) => {
                todo!("Received an open block")
            }
            BlockHolder::Change(_) => {
                todo!("Received a change block")
            }
            BlockHolder::State(state_block) => {
                let mut block = Block::from_state_block(&state_block);
                let block_hash: &BlockHash = {
                    block.calc_hash().unwrap();
                    block.hash().unwrap()
                };
                // dbg!(state_block);

                // # deduplication
                let already_exists = self.state.lock().await
                    .get_block_by_hash(block_hash).await?.is_some();

                // # signature validation
                let invalid_signature = || {
                    let valid_signature = block.verify_self_signature()
                        .map(|_| true)
                        .unwrap_or(false);
                    !valid_signature
                };

                if already_exists {
                    tracing::info!("Block {} already exists!", block_hash);
                } else if invalid_signature() {
                    tracing::info!("Block {} has invalid signature!", block);
                } else {
                    tracing::info!("Block {} will be added", block_hash);
                    self.state.lock().await.add_block(&block).await?;
                }
            }
        };
        //self.state.lock().await.add_block()

        // self.state.lock().await.add_block(&publish.0).await?;
        // todo!();

        Ok(())
    }

    pub async fn handle_confirm_req(
        &mut self,
        _header: &Header,
        _confirm_req: ConfirmReq,
    ) -> anyhow::Result<()> {
        // dbg!(confirm_req);
        Ok(())
    }

    pub async fn handle_confirm_ack(
        &mut self,
        _header: &Header,
        _confirm_ack: ConfirmAck,
    ) -> anyhow::Result<()> {
        // dbg!(confirm_ack);
        Ok(())
    }

    pub async fn handle_frontier_req(
        &mut self,
        _header: &Header,
        _frontier_req: FrontierReq,
    ) -> anyhow::Result<()> {
        // The rest of this connection will be a bunch of frontiers without any headers.
        self.frontier_stream = true;

        Ok(())
    }

    pub async fn handle_frontier_resp(
        &mut self,
        _frontier_resp: FrontierResp,
    ) -> anyhow::Result<()> {
        // dbg!(frontier_resp);
        // dbg!("----------------------------------------------------------------------");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Network;
    use std::sync::Arc;
    use tokio::sync::{Mutex};
    use std::net::{SocketAddr};
    use std::str::FromStr;
    use crate::node::{MemoryState};
    use crate::blocks::{StateBlock, Link};
    use crate::Rai;

    #[tokio::test]
    async fn should_not_add_block_if_signature_is_invalid() {
        let network = Network::Test;
        let state = MemoryState::new(network);
        let state = Arc::new(Mutex::new(state));
        let test_header = Header::new(network, MessageType::Handshake, Extensions::new());
        let test_socket_addr = SocketAddr::from_str("[::1]:1").unwrap();
        let (mut controller, _, _) = Controller::new_with_channels(network, state, test_socket_addr);
        let account = Public::from_str("570EDFC56651FBBC9AEFE5B0769DBD210614A0C0E6962F5CA0EA2FFF4C08A4B0").unwrap();
        let previous = BlockHash::from_str("C5C475D699CEED546FEC2E3A6C32B1544AB2C604D58D732B7D9BAB2D6A1E43E9").unwrap();
        let representative = Public::from_str("7194452B7997A9F5ABB2F434DB010CA18B5A2715D141F9CFA64A296B3EB4DCCD").unwrap();
        let signature = Some(Signature::zero());
        let state_block = StateBlock {
            account,
            previous,
            representative,
            balance: Rai(1344000000000000000000000000000),
            link: Link::Nothing,
            work: None,
            signature,
        };
        let mut block = Block::from_state_block(&state_block);
        block.calc_hash().unwrap();
        let block_holder = BlockHolder::State(state_block);
        controller.handle_publish(&test_header, Publish { block_holder }).await.unwrap();
        assert!(controller.state.lock().await.get_block_by_hash(block.hash().unwrap()).await.unwrap().is_none())
    }
}
