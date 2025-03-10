use crate::{
    constants::{ORDERBOOK_PDA_SEED, TRADER_PDA_SEED},
    state::{Orderbook, Trader},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateTrader<'info> {
    #[account(
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    #[account(
        init,
        payer = user,
        space = Trader::space(0),
        seeds = [TRADER_PDA_SEED, orderbook.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub trader: Account<'info, Trader>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateTrader<'info> {
    pub fn handler(ctx: Context<Self>) -> Result<()> {
        let trader = &mut ctx.accounts.trader;
        trader.bump = ctx.bumps.trader;
        trader.orderbook = ctx.accounts.orderbook.key();
        trader.user = ctx.accounts.user.key();

        Ok(())
    }
}
