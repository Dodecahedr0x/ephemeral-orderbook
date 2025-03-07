use anchor_lang::prelude::*;

#[error_code]
pub enum OrderbookError {
    #[msg("User does not exist in orderbook")]
    UnknownUser,
    #[msg("Not enough base tokens")]
    NotEnoughBaseTokens,
    #[msg("Not enough quote tokens")]
    NotEnoughQuoteTokens,
    #[msg("Order already matched")]
    AlreadyMatched,
    #[msg("Invalid order index")]
    InvalidOrderIndex,
    #[msg("Invalid order type")]
    InvalidOrderType,
    #[msg("Mismatching orders")]
    MismatchingOrders,
}
