mod blocks;
mod genesis;

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

    /// Update the representative weights based on this block being added to the network.
    pub async fn balance_rep_weights(&mut self, full_block: &FullBlock) -> anyhow::Result<()> {
        match full_block.block() {
            Block::Send(_) => {
                // TODO: Balance rep for send block
            }
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
    use crate::blocks::OpenBlock;
    use crate::blocks::SendBlock;
    use crate::node::state::MemoryState;
    use crate::{Seed, Work};
    use std::sync::Arc;

    async fn empty_lattice(network: Network) -> Controller {
        let state = Box::new(MemoryState::new(network));
        let mut controller = Controller::new(network, state);
        controller.init().await.unwrap();
        controller
    }

    #[tokio::test]
    async fn genesis() {
        let network = Network::Live;
        let genesis_full_block = network.genesis_block();
        let genesis_block = genesis_full_block.open_block().unwrap();

        let mut controller = empty_lattice(network).await;
        assert_eq!(
            controller
                .account_balance(&genesis_block.account)
                .await
                .unwrap()
                .expect("A balance"),
            Raw::max()
        );
    }

    #[tokio::test]
    async fn send_to_new_account() -> anyhow::Result<()> {
        let network = Network::Live;
        let genesis_full_block = network.genesis_block();
        let genesis_block = genesis_full_block.open_block()?;
        let new_key = Seed::random().derive(0);
        let new_account = new_key.to_public();

        let mut controller = empty_lattice(network).await;

        dbg!(&controller.state);

        // Send some Nano from the genesis account to the new one.
        let mut block = SendBlock::new(
            genesis_full_block.hash()?,
            new_account,
            Raw::from_mnano(1u128),
        )
        .into_full_block();

        dbg!(&controller.state);

        block.set_work(Work::zero()).unwrap(); // TODO: Real work
        block.sign(new_key).unwrap();
        controller.add_elected_block(&block).await?;

        dbg!(controller.state);

        // let new_account = OpenBlock::new()

        // let send_block = SendBlock::new()
        // controller.add_elected_block()
        Ok(())
    }
}
