use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Order {
    pub match_timestamp: Option<i64>,
    pub order_type: OrderType,
    pub price: u64,
    pub quantity: u64,
}

impl Order {
    pub const SPACE: usize = 8 + 9 + 1 + 8 + 8;
}

#[account]
pub struct Trader {
    pub bump: u8,
    pub orderbook: Pubkey,
    pub user: Pubkey,
    pub base_balance: u64,
    pub quote_balance: u64,
    // Uses a vec.
    // Inefficient, but simple.
    pub orders: Vec<Order>,
}

impl Trader {
    pub fn space(orders: usize) -> usize {
        8 + 1 + 32 + 32 + 8 + 8 + (4 + orders * Order::SPACE)
    }
}
