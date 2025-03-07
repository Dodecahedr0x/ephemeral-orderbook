use crate::{
    constants::ORDERBOOK_PDA_SEED,
    errors::OrderbookError,
    state::{Order, OrderType, Orderbook},
};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateOrderArgs {
    pub order: Order,
}

#[derive(Accounts)]
#[instruction(args: CreateOrderArgs)]
pub struct CreateOrder<'info> {
    #[account(
        mut,
        realloc = Orderbook::space(orderbook.orders.len() + 1, orderbook.user_balances.len()),
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

impl<'info> CreateOrder<'info> {
    pub fn handler(ctx: Context<Self>, args: CreateOrderArgs) -> Result<()> {
        let order = &args.order;
        if order.match_timestamp.is_some() {
            return err!(OrderbookError::AlreadyMatched);
        }

        let Some(user_balances) = ctx
            .accounts
            .orderbook
            .user_balances_mut(ctx.accounts.user.key)
        else {
            return err!(OrderbookError::UnknownUser);
        };

        // Remove assets from user balances
        if order.order_type == OrderType::Buy
            && user_balances.quote_balance < order.price * order.quantity
        {
            return err!(OrderbookError::NotEnoughQuoteTokens);
        } else {
            user_balances.quote_balance -= order.price * order.quantity;
        }
        if order.order_type == OrderType::Sell && user_balances.base_balance < order.quantity {
            return err!(OrderbookError::NotEnoughBaseTokens);
        } else {
            user_balances.base_balance -= order.quantity;
        }

        let orderbook = &mut ctx.accounts.orderbook;
        orderbook.orders.push(args.order);
        orderbook.orders.shrink_to_fit();

        Ok(())
    }
}
