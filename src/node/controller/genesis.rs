use crate::node::controller::Controller;
use crate::Raw;
use anyhow::Context;
use tracing::info;

impl Controller {
    pub async fn ensure_genesis(&mut self) -> anyhow::Result<()> {
        info!("Ensuring genesis");
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
}
