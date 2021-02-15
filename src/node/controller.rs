use crate::blocks::Block;
use crate::node::network::Network;
use crate::node::state::BoxedState;
use crate::{BlockHash, FullBlock, Public, Raw};
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
        let full_block = self.network.genesis_block();

        let added = self
            .add_elected_block(&full_block)
            .await
            .context("Adding genesis block")?;

        if added {
            // We know it's an open block.
            let block = full_block.open_block().context("Genesis")?;

            // The genesis block is an open block and we need to give it all the raw.
            self.state
                .set_account_balance(&block.account, &Raw::max())
                .await
                .context("Genesis account balance")?;
        }

        Ok(())
    }

    /// Add a block that has been deemed valid by ORV.
    ///
    /// Returns true if it was added.
    ///
    /// Before adding a block we need to make sure it:
    /// * Doesn't already exist.
    /// * Verify the work.
    /// * Verify the signature.
    ///
    /// After adding we need to update any representative weights.
    pub async fn add_elected_block(&mut self, block: &FullBlock) -> anyhow::Result<bool> {
        let context = || format!("Block {:?}", block);
        let block_hash = block.hash().with_context(context)?;

        // Block already exists, we can ignore this.
        if self
            .state
            .get_block_by_hash(&block_hash)
            .await
            .with_context(context)?
            .is_some()
        {
            return Ok(false);
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

        self.balance_rep_weights(block)
            .await
            .with_context(context)?;

        Ok(true)
    }

    /// Update the representative weights based on this block being added to the network.
    pub async fn balance_rep_weights(&mut self, full_block: &FullBlock) -> anyhow::Result<()> {
        match full_block.block() {
            // Block::Send(_) => {}
            // Block::Receive(_) => {}
            Block::Open(b) => {
                // Open blocks don't change in balance.
            }
            // Block::Change(_) => {}
            // Block::State(_) => {}
            _ => todo!(),
        };
        Ok(())
    }

    pub async fn account_balance(&mut self, account: &Public) -> anyhow::Result<Option<Raw>> {
        self.state.account_balance(account).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::state::MemoryState;
    use std::sync::Arc;

    #[tokio::test]
    async fn genesis() {
        let network = Network::Live;
        let genesis_full_block = network.genesis_block();
        let genesis_block = genesis_full_block.open_block().unwrap();
        let state = Box::new(MemoryState::new(network));
        let mut controller = Controller::new(network, state);
        controller.init().await.unwrap();
        assert_eq!(
            controller
                .account_balance(&genesis_block.account)
                .await
                .unwrap()
                .expect("A balance"),
            Raw::max()
        );
    }
}
