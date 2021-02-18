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
    use crate::encoding::FromHex;
    use crate::node::state::MemoryState;
    use crate::{Address, Seed, Work};
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
        let genesis_account =
            Address::from_str("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3")?
                .to_public();
        let dest_account =
            Address::from_str("nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo")?
                .to_public();

        let mut controller = empty_lattice(network).await;

        let send_block: FullBlock = serde_json::from_str(
            r#"
            {
                "type": "send",
                "previous": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
                "destination": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                "balance": "FD89D89D89D89D89D89D89D89D89D89D",
                "work": "3c82cc724905ee95",
                "signature": "5B11B17DB9C8FE0CC58CAC6A6EECEF9CB122DA8A81C6D3DB1B5EE3AB065AA8F8CB1D6765C8EB91B58530C5FF5987AD95E6D34BB57F44257E20795EE412E61600"
            }
        "#,
        )
        .unwrap();

        controller.add_elected_block(&send_block).await?;
        dbg!(&controller.state);

        let given = Raw::from(3271945835778254456378601994536232802u128);
        assert_eq!(
            controller.account_balance(&genesis_account).await?.unwrap(),
            Raw::max().checked_sub(&given).unwrap()
        );
        assert_eq!(
            controller.account_balance(&dest_account).await?.unwrap(),
            given
        );

        Ok(())
    }
}
