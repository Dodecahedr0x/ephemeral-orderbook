use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_and_undelegate_accounts};

use crate::{constants::ORDERBOOK_PDA_SEED, state::Orderbook};

#[commit]
#[derive(Accounts)]
pub struct UndelegateOrderbook<'info> {
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
}

impl<'info> UndelegateOrderbook<'info> {
    pub fn handler(ctx: Context<Self>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.orderbook.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;

        Ok(())
    }
}
