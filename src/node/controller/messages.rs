use super::Controller;
use crate::blocks::{Block, BlockHash, BlockHolder, BlockType, Previous, StateBlock};
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
use anyhow::{Context, Error};
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
            BlockHolder::State(mut state_block) => {
                Controller::state_block_handler(self, state_block)
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

    async fn state_block_handler(&self, state_block: StateBlock) -> anyhow::Result<()> {
        let block = Block::from_state_block(&state_block);
        // dbg!(state_block);
        let block = block; // give up mutability for safety

        if Controller::block_existed(self, block_hash).await? {
            tracing::info!("Block {} already exists!", block_hash);
        } else if block.verify_self_signature().is_err() {
            tracing::info!("Block {} has invalid signature!", block_hash);
        } else {
            let previous_block = Controller::block_by_hash(self, previous_hash).await?;

            match previous_block {
                None => {}
                Some(previous_block) => {
                    if previous_block.account() != block.account() {
                        tracing::info!(
                            "Block {} has previous belonging to another chain!",
                            block_hash
                        )
                    } else {
                        // Account already exists
                        match previous_block.previous() {
                            Previous::Block(prev_block_hash) => {
                                if Controller::block_exists(self, prev_block_hash).await? {
                                    let is_send = block.balance() < previous_block.balance();
                                    let mut decided_state_block = state_block;
                                    decided_state_block.decide_link_type(is_send);
                                    // let cannot_be_change_block = state_block.link
                                    // let is_receive = !is_send && !state_block.link
                                }
                            }
                            Previous::Open => {
                                tracing::info!("Block {} is opening an opened account!", block_hash)
                            }
                        }
                    }
                }
            }

            tracing::info!("Block {} will be added", block_hash);
            self.state.lock().await.add_block(&block).await?;
        }

        Ok(())
    }

    async fn process_valid_existing_state_block(
        &self,
        state_block: StateBlock,
        block: Block,
    ) -> anyhow::Result<()> {
        let previous_block = Controller::previous_as_account_info(self, state_block)
            .await
            .with_context(format!(
                "Block {} has previous which is not a frontier state block of the same account",
                block.hash()?
            ))?;
    }

    /// Returns the previous block if is a state block, belongs to the same account,
    /// and crucially, is a head block
    async fn previous_as_account_info(
        &self,
        state_block: StateBlock,
    ) -> anyhow::Result<StateBlock> {
        let previous_block = Controller::block_by_hash(self, previous_hash).await?;
        if let Some(previous_block) = previous_block {
            if previous_block.account() == state_block
                && *previous_block.is_head()
                && previous_block.block_type() == BlockType::State
            {
                return Ok(StateBlock::try_from(previous_block)?);
            }
        }
        Err(anyhow!("Cannot find previous block holding account info."))
    }

    /// Write block in the ledger
    async fn add_new_head_block(
        &self,
        block: &Block,
        previous_block: &Option<Block>,
    ) -> anyhow::Result<()> {
        // *start transaction*
        // 1 unmark previous as head block
        // 2 mark current as head block
        // 3 update previous block
        // 4 insert current block
        // *end transaction*
    }

    /// Shorthand for waiting a lock on the state and getting a block by hash
    async fn block_by_hash(&self, block_hash: &BlockHash) -> anyhow::Result<Option<Block>> {
        self.state.lock().await.get_block_by_hash(block_hash).await
    }

    /// Checks if the block exists in the database _or_ if it existed but was pruned
    async fn block_existed(&self, block_hash: &BlockHash) -> anyhow::Result<bool> {
        Ok(self
            .state
            .lock()
            .await
            .get_block_by_hash(block_hash)
            .await?
            .is_some())
    }

    /// For history nodes this has the same semantics as `Controller::block_existed`
    async fn block_exists(&self, block_hash: &BlockHash) -> anyhow::Result<bool> {
        Controller::block_existed(self, block_hash).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{Link, StateBlock};
    use crate::network::Network;
    use crate::node::MemoryState;
    use crate::Rai;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn should_not_add_block_if_signature_is_invalid() {
        let network = Network::Test;
        let state = MemoryState::new(network);
        let state = Arc::new(Mutex::new(state));
        let test_header = Header::new(network, MessageType::Handshake, Extensions::new());
        let test_socket_addr = SocketAddr::from_str("[::1]:1").unwrap();
        let (mut controller, _, _) =
            Controller::new_with_channels(network, state, test_socket_addr);
        let account =
            Public::from_str("570EDFC56651FBBC9AEFE5B0769DBD210614A0C0E6962F5CA0EA2FFF4C08A4B0")
                .unwrap();
        let previous =
            BlockHash::from_str("C5C475D699CEED546FEC2E3A6C32B1544AB2C604D58D732B7D9BAB2D6A1E43E9")
                .unwrap();
        let representative =
            Public::from_str("7194452B7997A9F5ABB2F434DB010CA18B5A2715D141F9CFA64A296B3EB4DCCD")
                .unwrap();
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
        let block_holder = BlockHolder::State(state_block);
        controller
            .handle_publish(&test_header, Publish { block_holder })
            .await
            .unwrap();
        assert!(controller
            .state
            .lock()
            .await
            .get_block_by_hash(block.hash().unwrap())
            .await
            .unwrap()
            .is_none())
    }
}
