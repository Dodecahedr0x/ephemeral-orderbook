use crate::{
    constants::ORDERBOOK_PDA_SEED,
    state::{Orderbook, UserBalances},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(
        mut,
        realloc = Orderbook::space(orderbook.orders.len(), orderbook.user_balances.len() + 1),
        realloc::payer = user,
        realloc::zero = true,
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateUser<'info> {
    pub fn handler(ctx: Context<Self>) -> Result<()> {
        let orderbook = &mut ctx.accounts.orderbook;

        // TODO: sort users by key
        orderbook
            .user_balances
            .push(UserBalances::new(ctx.accounts.user.key()));
        orderbook.user_balances.shrink_to_fit();

        Ok(())
    }
}
