use super::Peer;
use crate::blocks::{Block, BlockHash, BlockHolder, BlockType, Link, Previous, StateBlock};
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
use crate::{Difficulty, Public, Seed, Signature};
use anyhow::anyhow;
use anyhow::Context;
use std::convert::TryFrom;
use tracing::{debug, info, instrument, trace, warn};

impl Peer {
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

    #[instrument(skip(self, header, handshake))]
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
                .context("Verify recv handshake signature.")?;

            // Respond at the end because we mess with the header buffer.
            should_respond = ShouldRespond::Yes(public, signature);
        }

        if header.ext().is_response() {
            let response = handshake
                .response
                .context("response is None but is_response is True.")?;
            let public = response.public;
            let signature = response.signature;

            let cookie = &self
                .state
                .lock()
                .await
                .cookie_for_socket_addr(&self.peer_addr)
                .await
                .context("Could not lookup cookie for socket addr.")?;
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
            self.send_header(MessageType::Handshake, *Extensions::new().response())
                .await?;

            let response = HandshakeResponse::new(public, signature);
            self.send(&response)
                .await
                .context("Could not send response to peer.")?;
        }

        Ok(())
    }

    #[instrument(skip(self, _header, keepalive))]
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
        let _block = match publish.0 {
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
                self.state_block_handler(state_block).await?;
            }
        };

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
    /// Note: the returned block won't have Work, Amount or Signature
    async fn previous_as_account_info(
        &self,
        previous_block_hash: &BlockHash,
    ) -> anyhow::Result<Option<StateBlock>> {
        let previous_block = Peer::block_by_hash(self, previous_block_hash).await?;
        if let Some(previous_block) = previous_block {
            let is_head = self
                .get_latest_block(previous_block.account())
                .await?
                .map_or(false, |block| {
                    let block_hash = block.hash().context("hash for is_head").unwrap();
                    block_hash == previous_block_hash
                });

            dbg!(is_head);
            return if is_head && *previous_block.block_type() == BlockType::State {
                Ok(Some(StateBlock::try_from(previous_block)?))
            } else if !is_head && *previous_block.block_type() == BlockType::State {
                Err(anyhow!("The block referred as previous is not head!"))
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

    /// Actions to be performed to validate and store a state block
    /// TODO: this assumes we will never get a live epoch block
    async fn state_block_handler(&self, state_block: StateBlock) -> anyhow::Result<()> {
        // TODO: here there should be a check for epoch blocks
        if self.block_existed(&state_block.hash).await? {
            info!("Block {} already exists!", state_block)
        } else if state_block.verify_self_signature().is_err() {
            info!("Block {} has invalid signature!", state_block)
        } else {
            self.process_valid_existing_state_block(state_block).await?
        }
        Ok(())
    }

    async fn process_valid_existing_state_block(
        &self,
        state_block: StateBlock,
    ) -> anyhow::Result<()> {
        match &state_block.previous {
            Previous::Block(previous_hash) => {
                // Either wants to send, receive or change
                let maybe_previous_block = self.previous_as_account_info(previous_hash).await?;
                if let Some(previous_state_block) = maybe_previous_block {
                    self.process_block_with_previous(state_block, previous_state_block)
                        .await?
                } else {
                    info!("Block before {} not found!", state_block)
                }
            }
            Previous::Open => {
                todo!("Received an open sub-block")
            }
        }
        Ok(())
    }

    async fn process_block_with_previous(
        &self,
        mut state_block: StateBlock,
        previous_state_block: StateBlock,
    ) -> anyhow::Result<()> {
        let is_send = state_block.balance < previous_state_block.balance;
        let amount = if is_send {
            previous_state_block
                .balance
                .checked_sub(&state_block.balance)
        } else {
            state_block
                .balance
                .checked_sub(&previous_state_block.balance)
        };
        let amount = amount.ok_or(anyhow!("Could not calculate amount!"))?;
        state_block
            .set_link_type(is_send, amount)
            .context("Could not decide link type!")?;
        match state_block.link {
            Link::Nothing => {
                let _live_epoch_2_change_threshold = 0xfffffff800000000u64;
                todo!("Received a change sub-block")
            }
            Link::Source(_) => {
                let _live_epoch_2_receive_threshold = 0xfffffe0000000000u64;
                todo!("Received a receive sub-block")
            }
            Link::DestinationAccount(_) => self.process_good_send_sub_block(state_block).await,
            Link::Unsure(_) => {
                panic!("Unexpected error! Was `decide_link_type` called on this block?")
            }
        }
    }

    async fn process_good_send_sub_block(&self, send_block: StateBlock) -> anyhow::Result<()> {
        let live_epoch_2_send_threshold = 0xfffffff800000000u64;
        let block_difficulty = send_block
            .work
            .as_ref()
            .ok_or(anyhow!("Send sub-block {} has no work!", &send_block))?
            .difficulty_block_hash(&send_block.hash)?;
        let work_ok = block_difficulty >= Difficulty::new(live_epoch_2_send_threshold);
        if !work_ok {
            info!("Send sub-block {} has insufficient difficulty!", send_block);
            debug!(
                "Send sub-block {} had difficulty {}",
                send_block,
                block_difficulty.as_u64()
            );
        } else {
            self.store_block(&Block::from_state_block(&send_block))
                .await?
            // TODO: Update rep weight cache
            // TODO: Add to pending transactions
        }
        Ok(())
    }

    async fn store_block(&self, block: &Block) -> anyhow::Result<()> {
        // 1. if this block already exists, this operation is idempotent (but incurs in resource waste)
        // 2. if this block was added and rolled back this could generate an invalid state
        // 3. if we got a rollback request for this block and it didn't go through because it was missing
        //    this could generate an invalid state
        // 4. ???
        self.state.lock().await.add_block(block).await
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

    /// For history nodes this has the same semantics as `Peer::block_existed`
    /// Right now history nodes are not implemented so effectively there is no
    /// difference.
    async fn block_exists(&self, block_hash: &BlockHash) -> anyhow::Result<bool> {
        Peer::block_existed(self, block_hash).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{Link, Previous, StateBlock};
    use crate::network::Network;
    use crate::node::state::State;
    use crate::node::MemoryState;
    use crate::{Raw, Work};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn root_block() -> (StateBlock, Block) {
        let source = Link::Source(
            BlockHash::from_str("570EDFC56651FBBC9AEFE5B0769DBD210614A0C0E6962F5CA0EA2FFF4C08A4B0")
                .unwrap(),
        );
        let account =
            Public::from_str("570EDFC56651FBBC9AEFE5B0769DBD210614A0C0E6962F5CA0EA2FFF4C08A4B0")
                .unwrap();
        let representative =
            Public::from_str("7194452B7997A9F5ABB2F434DB010CA18B5A2715D141F9CFA64A296B3EB4DCCD")
                .unwrap();

        let root = StateBlock::new(account, Previous::Open, representative, Raw(500), source);
        let root_block = Block::from_state_block(&root);
        (root, root_block)
    }

    fn frontier_block() -> (StateBlock, Block) {
        let (_, root_block) = root_block();
        let destination = Link::DestinationAccount(
            Public::from_str("7194452B7997A9F5ABB2F434DB010CA18B5A2715D141F9CFA64A296B3EB4DCCD")
                .unwrap(),
        );
        let mut frontier = StateBlock::new(
            root_block.account().clone(),
            Previous::Block(root_block.hash().unwrap().clone()),
            root_block.representative().clone(),
            root_block.balance().checked_sub(&Raw(200)).unwrap(),
            destination,
        );
        frontier.work = Some(Work::from_str("8073a2031b9a3a6a").unwrap());
        let frontier_block = Block::from_state_block(&frontier);
        (frontier, frontier_block)
    }

    fn good_send_block() -> StateBlock {
        let (frontier_block, _) = frontier_block();
        frontier_block
    }

    fn bad_send_block() -> StateBlock {
        let (mut frontier_block, _) = frontier_block();
        frontier_block.work = Some(Work::from_str("baaaaaaaaaaaaaad").unwrap());
        frontier_block
    }

    async fn test_peer_with_blocks(blocks: &[&Block]) -> Peer {
        let network = Network::Test;
        let mut state_raw = MemoryState::new(network);
        for block in blocks {
            state_raw.add_block(*block).await.unwrap();
        }
        let test_socket_addr = SocketAddr::from_str("[::1]:1").unwrap();
        let state = Arc::new(Mutex::new(state_raw));
        let (peer, _, _) = Peer::new_with_channels(network, state, test_socket_addr);
        peer
    }

    #[tokio::test]
    #[should_panic(expected = "The block referred as previous is not head!")]
    async fn should_not_retrieve_previous_as_account_if_not_head() {
        let (_, root_block) = root_block();
        let (_, frontier_block) = frontier_block();
        let blocks = &[&root_block, &frontier_block];
        let peer = test_peer_with_blocks(blocks).await;

        Peer::previous_as_account_info(&peer, root_block.hash().unwrap())
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn should_retrieve_none_if_previous_not_existent() {
        let peer = test_peer_with_blocks(&[]).await;
        let (_, root_block) = root_block();

        let none = Peer::previous_as_account_info(&peer, root_block.hash().unwrap())
            .await
            .unwrap();
        assert!(none.is_none())
    }

    #[tokio::test]
    async fn should_retrieve_previous_as_account() {
        let (_, frontier_block) = frontier_block();
        let frontier = StateBlock::from(frontier_block.clone());
        let peer = test_peer_with_blocks(&[&frontier_block]).await;

        let frontier_result = Peer::previous_as_account_info(&peer, &frontier.hash)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(frontier, frontier_result)
    }

    #[tokio::test]
    async fn should_process_good_send_sub_block_when_block_is_good() {
        let peer = test_peer_with_blocks(&[]).await;
        let good_send_block = good_send_block();
        let good_send_block_hash = good_send_block.hash.clone();

        Peer::process_good_send_sub_block(&peer, good_send_block)
            .await
            .unwrap();

        let block_was_stored = Peer::block_exists(&peer, &good_send_block_hash)
            .await
            .unwrap();
        assert_eq!(block_was_stored, true)
    }

    #[tokio::test]
    async fn should_not_process_bad_send_sub_block_when_block_is_bad() {
        let peer = test_peer_with_blocks(&[]).await;
        let bad_send_block = bad_send_block();
        let bad_send_block_hash = bad_send_block.hash.clone();

        Peer::process_good_send_sub_block(&peer, bad_send_block)
            .await
            .unwrap();

        let block_was_stored = Peer::block_exists(&peer, &bad_send_block_hash)
            .await
            .unwrap();
        assert_eq!(block_was_stored, false)
    }

    #[tokio::test]
    async fn should_process_send_with_previous() {
        let (root, root_block) = root_block();
        let (frontier, _) = frontier_block();
        let peer = test_peer_with_blocks(&[&root_block]).await;

        Peer::process_block_with_previous(&peer, frontier.clone(), root)
            .await
            .unwrap();

        let block_was_stored = Peer::block_exists(&peer, &frontier.hash).await.unwrap();
        assert_eq!(block_was_stored, true)
    }

    #[tokio::test]
    async fn should_not_process_send_without_previous() {
        let (frontier, _) = frontier_block();
        let peer = test_peer_with_blocks(&[]).await;

        Peer::process_valid_existing_state_block(&peer, frontier.clone())
            .await
            .unwrap();

        let block_was_stored = Peer::block_exists(&peer, &frontier.hash).await.unwrap();
        assert_eq!(block_was_stored, false)
    }
}
