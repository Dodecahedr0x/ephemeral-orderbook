use crate::{
    constants::ORDERBOOK_PDA_SEED,
    errors::OrderbookError,
    state::{OrderType, Orderbook},
};
use anchor_lang::prelude::*;

// Assuming this is the format of oracle data:
// https://github.com/magicblock-labs/real-time-pricing-oracle/blob/main/program/ephemeral-oracle/tests/ephemeral-oracle.ts
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TemporalNumericValue {
    timestamp_ns: i64, // Using the same type as the solana clock
    quantized_value: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OracleData {
    symbol: String,
    id: Pubkey,
    temporal_numeric_value: TemporalNumericValue,
    publisher_merkle_root: [u8; 32],
    value_compute_alg_hash: [u8; 32],
    r: [u8; 32],
    s: [u8; 32],
    v: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MatchOrderArgs {
    pub oracle_data: OracleData,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub maker_index: u64, // Target a specific order to match for the maker
    pub taker_index: u64, // Target a specific order to match for the taker
}

#[derive(Accounts)]
#[instruction(args: MatchOrderArgs)]
pub struct MatchOrder<'info> {
    #[account(
        mut,
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    #[account(mut)]
    pub user: Signer<'info>,
}

impl<'info> MatchOrder<'info> {
    pub fn handler(ctx: Context<Self>, args: MatchOrderArgs) -> Result<()> {
        let orderbook = &mut ctx.accounts.orderbook;

        let maker_order = orderbook.orders[args.maker_index as usize].clone();
        if maker_order.order_type != OrderType::Sell {
            return err!(OrderbookError::InvalidOrderType);
        }
        if maker_order.owner != args.maker {
            return err!(OrderbookError::InvalidOrderOwner);
        }
        let taker_order = orderbook.orders[args.taker_index as usize].clone();
        if taker_order.order_type != OrderType::Buy {
            return err!(OrderbookError::InvalidOrderType);
        }
        if taker_order.owner != args.taker {
            return err!(OrderbookError::InvalidOrderOwner);
        }

        if maker_order.price > args.oracle_data.temporal_numeric_value.quantized_value
            || taker_order.price < args.oracle_data.temporal_numeric_value.quantized_value
            || maker_order.quantity != taker_order.quantity
        {
            return err!(OrderbookError::MismatchingOrders);
        }

        // The orders matched!
        // Assuming matching orders always have the same size
        let Some(maker) = orderbook.user_balances_mut(&args.maker) else {
            return err!(OrderbookError::UnknownUser);
        };
        maker.quote_balance += maker_order.price * maker_order.quantity;

        let Some(taker) = orderbook.user_balances_mut(&args.taker) else {
            return err!(OrderbookError::UnknownUser);
        };
        taker.base_balance += taker_order.quantity;

        orderbook.orders[args.taker_index as usize].match_timestamp =
            Some(args.oracle_data.temporal_numeric_value.timestamp_ns);
        orderbook.orders[args.maker_index as usize].match_timestamp =
            Some(args.oracle_data.temporal_numeric_value.timestamp_ns);

        if args.maker_index > args.taker_index {
            orderbook.orders.remove(args.maker_index as usize);
            orderbook.orders.remove(args.taker_index as usize);
        } else {
            orderbook.orders.remove(args.taker_index as usize);
            orderbook.orders.remove(args.maker_index as usize);
        }

        Ok(())
    }
}
