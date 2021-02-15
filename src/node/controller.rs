use crate::blocks::Block;
use crate::node::network::Network;
use crate::node::state::BoxedState;
use crate::{FullBlock, Public};
use anyhow::{anyhow, Context};

/// The controller handles the logic with handling and emitting messages, as well as time based
/// actions, peer management, etc.
struct Controller {
    network: Network,
    state: BoxedState,
}

impl Controller {
    pub fn new(network: Network, state: BoxedState) -> Self {
        Self { network, state }
    }

    /// Set up the genesis block if it hasn't already.
    pub async fn init(&mut self) -> anyhow::Result<()> {
        self.ensure_genesis().await.context("Ensuring genesis")?;
        Ok(())
    }

    pub async fn ensure_genesis(&mut self) -> anyhow::Result<()> {
        self.add_voted_block(&self.network.genesis_block())
            .await
            .context("Adding genesis block")?;

        Ok(())
    }

    /// Add a block that has been deemed valid by ORV.
    ///
    /// Before adding a block we need to make sure it:
    /// * Doesn't already exist.
    /// * Verify the work.
    /// * Verify the signature.
    ///
    /// After adding we need to update any representative weights.
    pub async fn add_voted_block(&mut self, block: &FullBlock) -> anyhow::Result<()> {
        let context = || format!("Block {:?}", block);
        let hash = block.hash().with_context(context)?;

        // Block already exists, we can ignore this.
        if self
            .state
            .get_block_by_hash(&hash)
            .await
            .with_context(context)?
            .is_some()
        {
            return Ok(());
        }

        let work = block.work();
        if work.is_none() {
            return Err(anyhow!("Work is missing from block")).with_context(context);
        }

        let signature = block.signature();
        if signature.is_none() {
            return Err(anyhow!("Signature is missing from block")).with_context(context);
        }

        // TODO: Verify work and signature

        self.state.add_block(block).await.with_context(context)?;

        self.update_rep_weight(block).await.with_context(context)?;

        Ok(())
    }

    /// Update the representative weights based on this block being added to the network.
    pub async fn update_rep_weight(&mut self, full_block: &FullBlock) -> anyhow::Result<()> {
        match full_block.block() {
            Block::Send(_) => {}
            Block::Receive(_) => {}
            Block::Open(b) => {}
            Block::Change(_) => {}
            Block::State(_) => {}
        };
        todo!()
    }
}
