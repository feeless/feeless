mod blocks;
mod genesis;

use crate::node::network::Network;
use crate::node::state::BoxedState;
use crate::{Block, Public, Raw};
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
    pub async fn balance_rep_weights(&mut self, _full_block: &Block) -> anyhow::Result<()> {
        // match full_block.block() {
        //     Block::Send(_) => {
        //         // TODO: Balance rep for send block
        //     }
        //     // Block::Receive(_) => {}
        //     Block::Open(_b) => {
        //         // Open blocks don't change in balance.
        //     }
        //     // Block::Change(_) => {}
        //     // Block::State(_) => {}
        //     _ => todo!(),
        // };
        // Ok(())
        todo!()
    }

    pub async fn account_balance(&self, account: &Public) -> anyhow::Result<Raw> {
        let context = || anyhow!("Account balance for {:?}", account);
        let block = self.get_latest_block(account).await.with_context(context)?;

        match block {
            Some(block) => Ok(block.balance().to_owned()),
            None => Ok(Raw::zero()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::blocks::{OpenBlock, SendBlock};
    use crate::encoding::FromHex;
    use crate::node::state::MemoryState;
    use crate::{Address, BlockHash};

    async fn empty_lattice(network: Network) -> Controller {
        let state = Box::new(MemoryState::new(network));
        let mut controller = Controller::new(network, state);
        controller.init().await.unwrap();
        controller
    }

    #[tokio::test]
    async fn genesis() {
        let network = Network::Live;
        let genesis = network.genesis_block();

        let controller = empty_lattice(network).await;
        dbg!(&controller.state);
        assert_eq!(
            controller
                .get_latest_block(genesis.account())
                .await
                .unwrap()
                .unwrap()
                .balance(),
            &Raw::max()
        );
    }

    /// Genesis Account: genesis (Open) -> gen_send (Send)
    /// Landing Account:                -> land_open (Open) -> land_send (Send)
    #[tokio::test]
    async fn send_then_recv_to_new_account() {
        let network = Network::Live;
        let genesis = network.genesis_block();

        let landing_account =
            Address::from_str("nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo")
                .unwrap()
                .to_public();

        let mut controller = empty_lattice(network).await;

        let gen_send: SendBlock = serde_json::from_str(
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

        // TODO: This should be done somewhere (the controller?
        // e.g. controller.validate_send_block() or controller.fill_send_block()
        let mut block: Block =
            Block::from_send_block(&gen_send, genesis.account(), genesis.representative());
        block.calc_hash().unwrap();

        controller.add_elected_block(&block).await.unwrap();

        let given = Raw::from(3271945835778254456378601994536232802u128);

        let genesis_balance = Raw::max().checked_sub(&given).unwrap();

        // The genesis account has a reduced amount because they've created a send block.
        assert_eq!(
            controller
                .account_balance(&genesis.account())
                .await
                .unwrap(),
            genesis_balance
        );

        // Account isn't opened yet so it's empty.
        assert_eq!(
            controller.account_balance(&landing_account).await.unwrap(),
            Raw::zero()
        );

        // TODO: Check pending balance of landing account.

        // A real open block to the "Landing" account.
        // `type` is ignored here, but just left it in as it's ignored.
        let land_open: OpenBlock = serde_json::from_str(
            r#"{
                "type": "open",
                "source": "A170D51B94E00371ACE76E35AC81DC9405D5D04D4CEBC399AEACE07AE05DD293",
                "representative": "nano_1awsn43we17c1oshdru4azeqjz9wii41dy8npubm4rg11so7dx3jtqgoeahy",
                "account": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                "work": "e997c097a452a1b1",
                "signature": "E950FFDF0C9C4DAF43C27AE3993378E4D8AD6FA591C24497C53E07A3BC80468539B0A467992A916F0DDA6F267AD764A3C1A5BDBD8F489DFAE8175EEE0E337402"
            }"#,
        ).unwrap();
        let mut land_open = Block::from_open_block(&land_open, &BlockHash::zero(), &given);
        land_open.calc_hash().unwrap();
        assert_eq!(
            land_open.hash().unwrap(),
            &BlockHash::from_hex(
                "90D0C16AC92DD35814E84BFBCC739A039615D0A42A76EF44ADAEF1D99E9F8A35"
            )
            .unwrap()
        );

        controller.add_elected_block(&land_open).await.unwrap();
        dbg!(&controller.state);

        assert_eq!(
            controller.account_balance(&landing_account).await.unwrap(),
            given
        );
    }
}
