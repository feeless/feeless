use crate::blocks::Block;
use crate::node::controller::Controller;
use crate::{FullBlock, Public, Raw};
use anyhow::{anyhow, Context};
use tracing::debug;

struct AccountDelta {
    from: Public,
    to: Public,
    amount: Raw,
}

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
        // In reality this shouldn't even happen so it should be a panic.
        // This function should only have the chance to be called once per block.
        if self
            .state
            .get_block_by_hash(&block_hash)
            .await
            .with_context(context)?
            .is_some()
        {
            return Ok(false);
        }

        let from_account = self.account_for_block(&block).await?;
        dbg!(&from_account);

        let context = || format!("Block {:?} Account: {:?}", block, &from_account);
        if !block
            .verify_signature(&from_account)
            .with_context(context)?
        {
            return Err(anyhow!("Incorrect signature")).with_context(context);
        }

        let work = block.work();
        if work.is_none() {
            return Err(anyhow!("Work is missing from block")).with_context(context);
        }
        // TODO: Verify work

        // TODO: For now just assume this is a send block
        if let Ok(send_block) = block.send_block() {
            let to_account = &send_block.destination;
            let to_balance = self
                .state
                .recv_account_balance(&to_account)
                .await
                .with_context(context)?;

            let old_balance = self.state.recv_account_balance(&from_account).await?;
            let from_new_balance = &send_block.balance;
            if from_new_balance >= &old_balance {
                return Err(anyhow!("Can not increase balance in a send block"))
                    .with_context(context);
            }
            let amount = old_balance
                .checked_sub(from_new_balance)
                .ok_or_else(|| {
                    anyhow!(
                        "Subtracting old_balance {:?} and from_new_balance {:?}",
                        old_balance,
                        from_new_balance
                    )
                })
                .with_context(context)?;

            // The account is lowering its balance on both sent and recv balances.
            self.state
                .set_sent_account_balance(&from_account, &from_new_balance)
                .await?;
            self.state
                .set_recv_account_balance(&from_account, &from_new_balance)
                .await?;

            // The receiving "sent account" is reduced, but not the "recv account" until a recv
            // block is confirmed.
            let to_new_balance = to_balance
                .checked_add(&amount)
                .ok_or_else(|| {
                    anyhow!("Adding to_balance {:?} and amount {:?}", to_balance, amount)
                })
                .with_context(context)?;

            self.state
                .set_sent_account_balance(&to_account, &to_new_balance)
                .await?;
        } else {
            todo!();
        }

        self.state
            .add_block(&from_account, block)
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
