use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_and_undelegate_accounts};

use crate::{constants::TRADER_PDA_SEED, state::Trader};

#[commit]
#[derive(Accounts)]
pub struct UndelegateTrader<'info> {
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [TRADER_PDA_SEED, trader.orderbook.as_ref(), payer.key().as_ref()],
        bump,
    )]
    pub trader: Account<'info, Trader>,
}

impl<'info> UndelegateTrader<'info> {
    pub fn handler(ctx: Context<Self>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.trader.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;

        Ok(())
    }
}
