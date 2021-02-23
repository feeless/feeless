use crate::blocks::BlockType;
use crate::node::controller::Controller;
use crate::{Block, Public, Raw};
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
    /// * Handle the specific block type appropriately.
    ///
    /// After adding we need to update any representative weights.
    pub async fn add_elected_block(&mut self, block: &Block) -> anyhow::Result<()> {
        debug!("Adding elected block {:?}", &block);
        let context = || format!("Block {:?}", &block);
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
            return Err(anyhow!("Block already exists")).with_context(context);
        }

        let context = || format!("Block {:?}", block);
        if !block
            .verify_signature(&block.account())
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
        match block.block_type() {
            BlockType::Send => {
                // let to_account = &block.destination().with_context(context)?;
                // let from_new_balance = block.balance();
                // if from_new_balance >= &old_balance {
                //     return Err(anyhow!("Can not increase balance in a send block"))
                //         .with_context(context);
                // }
                // let amount = old_balance
                //     .checked_sub(from_new_balance)
                //     .ok_or_else(|| {
                //         anyhow!(
                //             "Subtracting old_balance {:?} and from_new_balance {:?}",
                //             old_balance,
                //             from_new_balance
                //         )
                //     })
                //     .with_context(context)?;
                //
                // // The account is lowering its balance on both sent and recv balances.
                // self.state
                //     .set_sent_account_balance(&block.account(), &from_new_balance)
                //     .await?;
                // self.state
                //     .set_recv_account_balance(&block.account(), &from_new_balance)
                //     .await?;
                //
                // // The receiving "sent account" is reduced, but not the "recv account" until a recv
                // // block is confirmed.
                // let to_new_balance = to_balance
                //     .checked_add(&amount)
                //     .ok_or_else(|| {
                //         anyhow!("Adding to_balance {:?} and amount {:?}", to_balance, amount)
                //     })
                //     .with_context(context)?;
                //
                // self.state
                //     .set_sent_account_balance(&to_account, &to_new_balance)
                //     .await?;
                todo!()
            }
            BlockType::Open => {
                dbg!(block);

                // If the block is the genesis block, we basically just trust the balance.
                if block.is_genesis(&self.network)? {
                    // self.state
                    //     .set_sent_account_balance(block.account(), block.balance());
                    // self.state
                    //     .set_recv_account_balance(block.account(), block.balance());
                } else {
                    todo!();
                }

                // let to_account = block.account();
                // // If the block is the genesis block, we give Raw::max instead of the amount from the
                // // previous block.
                // let add_amount = if *block == self.network.genesis_block() {
                //     Raw::max()
                // } else {
                //     let send_block = self
                //         .state
                //         .get_block_by_hash(&open_block.source)
                //         .await
                //         .with_context(context)?
                //         .ok_or_else(|| anyhow!("Open block has a reference to a non existent block"))?
                //         .send_block()
                //         .context("Open block is referencing a block that is not a send block")?;
                //
                //     let to_balance = self
                //         .state
                //         .recv_account_balance(&to_account)
                //         .await
                //         .with_context(context)?;
                // };
                // todo!();
            }
            _ => todo!(),
        }

        self.state
            .add_block(&block.account(), block)
            .await
            .with_context(context)?;

        // self.balance_rep_weights(block)
        //     .await
        //     .with_context(context)?;

        Ok(())
    }

    pub async fn get_latest_block(&mut self, account: &Public) -> anyhow::Result<Option<Block>> {
        let block_hash = self
            .state
            .get_latest_block_hash_for_account(account)
            .await
            .with_context(|| format!("Account: {:?}", account))?
            .ok_or_else(|| anyhow!("No block found for account: {:?}", account))?;

        Ok(self
            .state
            .get_block_by_hash(&block_hash)
            .await
            .with_context(|| {
                format!(
                    "Could not get block for latest hash for account: {:?}",
                    account
                )
            })?)
    }
}
