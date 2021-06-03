use crate::blocks::{Block, BlockHash, BlockType, Previous};
use crate::node::messages::confirm_ack::{Confirm, ConfirmAck};
use crate::node::peer::Peer;
use crate::{Public, Raw, Signature};
use anyhow::{anyhow, Context};
use tracing::{debug, instrument, warn};

struct AccountDelta {
    from: Public,
    to: Public,
    amount: Raw,
}

impl Peer {
    #[instrument(skip(self))]
    pub async fn add_vote(&mut self, confirm_ack: &ConfirmAck) -> anyhow::Result<()> {
        let context = || format!("Adding vote {:?}", &confirm_ack);

        let hashes = if let Confirm::VoteByHash(hashes) = &confirm_ack.confirm {
            hashes
        } else {
            return Err(anyhow!("Confirm::Block not implemented")).with_context(context);
        };

        for hash in hashes {
            self.validate_vote(hash, &confirm_ack.account, &confirm_ack.signature)
                .await
                .with_context(context)?;

            self.state
                .lock()
                .await
                .add_vote(hash, &confirm_ack.account)
                .await
                .with_context(context)?;

            // self.check_votes(&confirm_ack.confirm)
            //     .await
            //     .with_context(context)?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn validate_vote(
        &mut self,
        hash: &BlockHash,
        representative: &Public,
        signature: &Signature,
    ) -> anyhow::Result<()> {
        warn!("TODO validate vote");
        Ok(())
    }

    /// Add a block that has been deemed valid by ORV.
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
            .lock()
            .await
            .get_block_by_hash(&block_hash)
            .await
            .with_context(context)?
            .is_some()
        {
            return Err(anyhow!("Block already exists")).with_context(context);
        }

        let context = || format!("Block {:?}", block);
        block
            .verify_signature(&block.account())
            .context("Incorrect signature")
            .with_context(context)?;

        let work = block.work();
        if work.is_none() {
            return Err(anyhow!("Work is missing from block")).with_context(context);
        }
        // TODO: Verify work

        // TODO: For now just assume this is a send block
        match block.block_type() {
            BlockType::Send => {
                dbg!(block);

                let previous_hash = match block.previous() {
                    Previous::Block(h) => h,
                    Previous::Open => {
                        return Err(anyhow!("Send block has a blank previous block hash"))
                            .with_context(context)
                    }
                };

                let prev_block = self
                    .state
                    .lock()
                    .await
                    .get_block_by_hash(previous_hash)
                    .await
                    .context("Previous block")
                    .with_context(context)?
                    .ok_or_else(|| anyhow!("Could not find previous block"))
                    .with_context(context)?;
                let prev_balance = prev_block.balance();

                if block.balance() >= prev_balance {
                    return Err(anyhow!(
                        "Can not increase balance in a send block. Prev: {:?}",
                        prev_block
                    ))
                    .with_context(context);
                }

                let _to_account = block.destination().with_context(context)?;
                let _amount = prev_balance
                    .checked_sub(block.balance())
                    .ok_or_else(|| {
                        anyhow!(
                            "Subtracting prev_balance {:?} and new balance {:?}",
                            prev_balance,
                            block.balance()
                        )
                    })
                    .with_context(context)?;
            }
            BlockType::Open => {
                dbg!(block);

                // If the block is the genesis block, we basically just trust the balance.
                if !block.is_genesis(&self.network)? {
                    // TODO: Make sure the balance in the open block matches the amount in the
                    // send block.
                }
            }
            _ => todo!(),
        }

        self.state
            .lock()
            .await
            .add_block(block)
            .await
            .with_context(context)?;

        // self.balance_rep_weights(block)
        //     .await
        //     .with_context(context)?;

        Ok(())
    }

    pub async fn get_latest_block(&self, account: &Public) -> anyhow::Result<Option<Block>> {
        let block_hash = self
            .state
            .lock()
            .await
            .get_latest_block_hash_for_account(account)
            .await
            .with_context(|| format!("Account: {:?}", account))?;
        let block_hash = match block_hash {
            Some(b) => b,
            None => return Ok(None),
        };

        Ok(self
            .state
            .lock()
            .await
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
