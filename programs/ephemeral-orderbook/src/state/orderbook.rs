use anchor_lang::prelude::*;

#[account]
pub struct Orderbook {
    pub bump: u8,
    pub id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

impl Orderbook {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + 32;
}
