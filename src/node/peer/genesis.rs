use crate::node::peer::Peer;

use anyhow::Context;
use tracing::info;

impl Peer {
    pub async fn ensure_genesis(&mut self) -> anyhow::Result<()> {
        info!("Ensuring genesis");
        let mut block = self.network.genesis_block();

        self.add_elected_block(&mut block)
            .await
            .context("Adding genesis block")?;

        Ok(())
    }
}
