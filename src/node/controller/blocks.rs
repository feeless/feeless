use crate::blocks::Block;
use crate::node::controller::Controller;
use crate::{FullBlock, Public};
use anyhow::{anyhow, Context};
use tracing::debug;

impl Controller {
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
    ///
    /// Thinking out loud:
    ///
    /// * A send block:
    ///   * Grab the parent
    ///   * Check the balance is lower than before from the parent
    pub async fn add_elected_block(&mut self, block: &FullBlock) -> anyhow::Result<bool> {
        debug!("Adding elected block {:?}", block);
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

        let account = self.account_for_block(&block).await?;
        dbg!(&account);

        let context = || format!("Block {:?} Account: {:?}", block, &account);
        if !block.verify_signature(&account).with_context(context)? {
            return Err(anyhow!("Incorrect signature")).with_context(context);
        }

        let work = block.work();
        if work.is_none() {
            return Err(anyhow!("Work is missing from block")).with_context(context);
        }

        // // TODO: Check if the sender block exists and has enough funds.
        // let parent = self.find_parent_block(block).await.with_context(context)?;
        // // if parent.is_none() &&
        // // TODO: dont unwrap
        // let parent = parent.unwrap();
        // TODO: Verify work and signature

        // TODO: For now just assume this is a send block
        if let Ok(send_block) = block.send_block() {
            self.state
                .set_account_balance(&account, &send_block.balance)
                .await?;
        }

        self.state
            .add_block(&account, block)
            .await
            .with_context(context)?;

        self.balance_rep_weights(block)
            .await
            .with_context(context)?;

        Ok(true)
    }

    pub async fn account_for_block(&mut self, block: &FullBlock) -> anyhow::Result<Public> {
        let account = match block.block() {
            Block::Open(o) => o.source.to_owned(),
            _ => {
                let previous = block
                    .previous()
                    .expect("A non open block doesn't have a previous block hash");
                // TODO: Handle missing account
                self.state
                    .account_for_block_hash(&previous)
                    .await?
                    .expect("TODO: Handle missing block")
            }
        };

        Ok(account)
    }

    /// Return the parent block of this block.
    ///
    /// This might need a few hits in the database, depending on the block.
    pub async fn find_parent_block(
        &mut self,
        _block: &FullBlock,
    ) -> anyhow::Result<Option<FullBlock>> {
        // let maybe_block = if let Some(block_hash) = block.parent_hash() {
        //     self.find_block_by_hash(block_hash).await?
        // } else {
        //     todo!();
        //     // self.find_block_by_destination().await?
        // };

        todo!()
    }
}
