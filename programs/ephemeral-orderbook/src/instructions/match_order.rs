use crate::{
    constants::{ORDERBOOK_PDA_SEED, TRADER_PDA_SEED},
    errors::OrderbookError,
    state::{OrderType, Orderbook, Trader},
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
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    #[account(
        mut,
        seeds = [TRADER_PDA_SEED, orderbook.key().as_ref(), args.maker.as_ref()],
        bump = maker.bump,
        has_one = orderbook,
    )]
    pub maker: Box<Account<'info, Trader>>,
    #[account(
        mut,
        seeds = [TRADER_PDA_SEED, orderbook.key().as_ref(), args.taker.as_ref()],
        bump = taker.bump,
        has_one = orderbook,
    )]
    pub taker: Box<Account<'info, Trader>>,
}

impl<'info> MatchOrder<'info> {
    pub fn handler(ctx: Context<Self>, args: MatchOrderArgs) -> Result<()> {
        let maker = &mut ctx.accounts.maker;
        let taker = &mut ctx.accounts.taker;

        let maker_order = maker.orders[args.maker_index as usize].clone();
        if maker_order.order_type != OrderType::Sell {
            return err!(OrderbookError::InvalidOrderType);
        }
        let taker_order = taker.orders[args.taker_index as usize].clone();
        if taker_order.order_type != OrderType::Buy {
            return err!(OrderbookError::InvalidOrderType);
        }

        if maker_order.price > args.oracle_data.temporal_numeric_value.quantized_value
            || taker_order.price < args.oracle_data.temporal_numeric_value.quantized_value
            || maker_order.quantity != taker_order.quantity
        {
            return err!(OrderbookError::MismatchingOrders);
        }

        // The orders matched!
        // Assuming matching orders always have the same size
        maker.quote_balance += maker_order.price * maker_order.quantity;
        taker.base_balance += taker_order.quantity;

        maker.orders[args.maker_index as usize].match_timestamp =
            Some(args.oracle_data.temporal_numeric_value.timestamp_ns);
        taker.orders[args.taker_index as usize].match_timestamp =
            Some(args.oracle_data.temporal_numeric_value.timestamp_ns);

        maker.orders.remove(args.maker_index as usize);
        taker.orders.remove(args.taker_index as usize);

        Ok(())
    }
}
