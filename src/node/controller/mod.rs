mod blocks;
mod genesis;

use crate::blocks::Block;
use crate::node::network::Network;
use crate::node::state::BoxedState;
use crate::{FullBlock, Public, Raw};
use anyhow::Context;

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
            Block::Open(_b) => {
                // Open blocks don't change in balance.
            }
            // Block::Change(_) => {}
            // Block::State(_) => {}
            _ => todo!(),
        };
        Ok(())
    }

    pub async fn sent_account_balance(&mut self, account: &Public) -> anyhow::Result<Raw> {
        Ok(self
            .state
            .sent_account_balance(account)
            .await
            .unwrap_or(Raw::zero()))
    }

    pub async fn recv_account_balance(&mut self, account: &Public) -> anyhow::Result<Raw> {
        Ok(self
            .state
            .recv_account_balance(account)
            .await
            .unwrap_or(Raw::zero()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::encoding::FromHex;
    use crate::node::state::MemoryState;
    use crate::{Address, BlockHash};

    async fn empty_lattice(network: Network) -> Controller {
        let state = Box::new(MemoryState::new(network));
        let mut controller = Controller::new(network, state);
        controller.init().await.unwrap();
        controller
    }

    async fn assert_sent_balance(controller: &mut Controller, account: &Public, raw: &Raw) {
        assert_eq!(
            &controller.sent_account_balance(&account).await.unwrap(),
            raw
        );
    }
    async fn assert_recv_balance(controller: &mut Controller, account: &Public, raw: &Raw) {
        assert_eq!(
            &controller.recv_account_balance(&account).await.unwrap(),
            raw
        );
    }

    #[tokio::test]
    async fn genesis() {
        let network = Network::Live;
        let genesis_full_block = network.genesis_block();
        let genesis_block = genesis_full_block.open_block().unwrap();

        let mut controller = empty_lattice(network).await;
        assert_sent_balance(&mut controller, &genesis_block.account, &Raw::max()).await;
        assert_recv_balance(&mut controller, &genesis_block.account, &Raw::max()).await;
    }

    #[tokio::test]
    async fn send_then_recv_to_new_account() {
        let network = Network::Live;
        let genesis_account =
            Address::from_str("nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3")
                .unwrap()
                .to_public();
        let dest_account =
            Address::from_str("nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo")
                .unwrap()
                .to_public();

        let mut controller = empty_lattice(network).await;

        let send_block: FullBlock = serde_json::from_str(
            r#"{
                "type": "send",
                "previous": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
                "destination": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                "balance": "FD89D89D89D89D89D89D89D89D89D89D",
                "work": "3c82cc724905ee95",
                "signature": "5B11B17DB9C8FE0CC58CAC6A6EECEF9CB122DA8A81C6D3DB1B5EE3AB065AA8F8CB1D6765C8EB91B58530C5FF5987AD95E6D34BB57F44257E20795EE412E61600"
            }"#,
        )
        .unwrap();

        controller.add_elected_block(&send_block).await.unwrap();

        let given = Raw::from(3271945835778254456378601994536232802u128);

        // Check the sender's account on both received and sent balances.
        let genesis_balance = Raw::max().checked_sub(&given).unwrap();
        assert_sent_balance(&mut controller, &genesis_account, &genesis_balance).await;
        assert_recv_balance(&mut controller, &genesis_account, &genesis_balance).await;

        // Only the sent block exists.
        assert_sent_balance(&mut controller, &dest_account, &given).await;

        // The account has no receive funds because there is no open/receive block added yet.
        assert_eq!(
            controller
                .recv_account_balance(&dest_account)
                .await
                .unwrap(),
            Raw::zero()
        );

        // A real open block to the "Landing" account.
        let open_block: FullBlock = serde_json::from_str(
            r#"{
                "type": "open",
                "source": "A170D51B94E00371ACE76E35AC81DC9405D5D04D4CEBC399AEACE07AE05DD293",
                "representative": "nano_1awsn43we17c1oshdru4azeqjz9wii41dy8npubm4rg11so7dx3jtqgoeahy",
                "account": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                "work": "e997c097a452a1b1",
                "signature": "E950FFDF0C9C4DAF43C27AE3993378E4D8AD6FA591C24497C53E07A3BC80468539B0A467992A916F0DDA6F267AD764A3C1A5BDBD8F489DFAE8175EEE0E337402"
            }"#,
        ).unwrap();
        assert_eq!(
            open_block.hash().unwrap(),
            BlockHash::from_hex("90D0C16AC92DD35814E84BFBCC739A039615D0A42A76EF44ADAEF1D99E9F8A35")
                .unwrap()
        );

        controller.add_elected_block(&open_block).await.unwrap();
        dbg!(&controller.state);

        assert_sent_balance(&mut controller, &dest_account, &given).await;
        assert_recv_balance(&mut controller, &dest_account, &given).await;
    }
}
