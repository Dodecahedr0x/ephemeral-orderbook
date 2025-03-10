use crate::{
    constants::{ORDERBOOK_PDA_SEED, TRADER_PDA_SEED},
    errors::OrderbookError,
    state::{Order, OrderType, Orderbook, Trader},
};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateOrderArgs {
    pub order: Order,
}

#[derive(Accounts)]
#[instruction(args: CreateOrderArgs)]
pub struct CreateOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    #[account(
        mut,
        realloc = Trader::space(trader.orders.len() + 1),
        realloc::payer = user,
        realloc::zero = true,
        seeds = [TRADER_PDA_SEED, orderbook.key().as_ref(), user.key().as_ref()],
        bump = trader.bump,
        has_one = orderbook,
        has_one = user,
    )]
    pub trader: Box<Account<'info, Trader>>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOrder<'info> {
    pub fn handler(ctx: Context<Self>, args: CreateOrderArgs) -> Result<()> {
        let order = &args.order;
        if order.match_timestamp.is_some() {
            return err!(OrderbookError::AlreadyMatched);
        }

        let trader = &mut ctx.accounts.trader;

        // Remove assets from user balances
        if order.order_type == OrderType::Buy {
            require_gte!(
                trader.quote_balance,
                order.price * order.quantity,
                OrderbookError::NotEnoughQuoteTokens
            );

            trader.quote_balance -= order.price * order.quantity;
        }
        if order.order_type == OrderType::Sell {
            require_gte!(
                trader.base_balance,
                order.quantity,
                OrderbookError::NotEnoughBaseTokens
            );

            trader.base_balance -= order.quantity;
        }

        trader.orders.push(args.order);
        trader.orders.shrink_to_fit();

        Ok(())
    }
}
