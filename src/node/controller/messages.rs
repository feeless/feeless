use super::Controller;
use crate::blocks::{Block, BlockHash, BlockType, StateBlock};
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
use anyhow::anyhow;
use anyhow::Context;
use std::convert::TryFrom;
use tracing::{debug, instrument, trace, warn};

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
        _publish: Publish,
    ) -> anyhow::Result<()> {
        // dbg!(publish);

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

    /// Returns the previous block if is a head block AND is a state_block
    async fn previous_as_account_info(
        &self,
        previous_block_hash: &BlockHash,
    ) -> anyhow::Result<Option<StateBlock>> {
        let previous_block = Controller::block_by_hash(self, previous_block_hash).await?;
        if let Some(previous_block) = previous_block {
            return if *previous_block.is_head() && *previous_block.block_type() == BlockType::State
            {
                Ok(Some(StateBlock::try_from(previous_block)?))
            } else {
                Err(anyhow!(
                    "Previous block existed but is not currently supported!"
                ))
                // in future versions this should build the account information by
                // backtracing. No attack vector is possible here to make it slower
                // because these blocks are not supported anymore and should be
                // discarded.
            };
        }
        Ok(None)
    }

    /// Shorthand for waiting a lock on the state and getting a block by hash
    async fn block_by_hash(&self, block_hash: &BlockHash) -> anyhow::Result<Option<Block>> {
        self.state.lock().await.get_block_by_hash(block_hash).await
    }
}
