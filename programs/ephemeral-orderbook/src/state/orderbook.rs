use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Order {
    pub id: Pubkey,
    pub orderbook: Pubkey,
    pub owner: Pubkey,
    pub match_timestamp: Option<i64>,
    pub order_type: OrderType,
    pub price: u64,
    pub quantity: u64,
}

impl Order {
    pub const SPACE: usize = 8 + 32 + 32 + 1 + 32 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserBalances {
    pub user: Pubkey,
    pub base_balance: u64,
    pub quote_balance: u64,
}

impl UserBalances {
    pub const SPACE: usize = 8 + 32 + 8 + 8;

    pub fn new(user: Pubkey) -> Self {
        Self {
            user,
            base_balance: 0,
            quote_balance: 0,
        }
    }
}

#[account]
pub struct Orderbook {
    pub bump: u8,
    pub id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    // Uses vecs.
    // Inefficient, but simple.
    pub orders: Vec<Order>,
    pub user_balances: Vec<UserBalances>,
}

impl Orderbook {
    pub fn space(orders: usize, users: usize) -> usize {
        8 + 1 + 32 + 32 + 32 + (4 + orders * Order::SPACE) + (4 + users * UserBalances::SPACE)
    }

    pub fn user_balances(&self, user: &Pubkey) -> Option<&UserBalances> {
        self.user_balances.iter().find(|ub| ub.user == *user)
    }

    pub fn user_balances_mut(&mut self, user: &Pubkey) -> Option<&mut UserBalances> {
        self.user_balances.iter_mut().find(|ub| ub.user == *user)
    }
}
