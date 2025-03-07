use crate::{constants::ORDERBOOK_PDA_SEED, state::Orderbook};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeOrderbookArgs {
    pub id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: InitializeOrderbookArgs)]
pub struct InitializeOrderbook<'info> {
    #[account(
        init,
        payer = user,
        space = Orderbook::space(0, 0),
        seeds = [ORDERBOOK_PDA_SEED, args.id.as_ref()],
        bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeOrderbook<'info> {
    pub fn handler(ctx: Context<Self>, args: InitializeOrderbookArgs) -> Result<()> {
        let orderbook = &mut ctx.accounts.orderbook;
        orderbook.bump = ctx.bumps.orderbook;
        orderbook.id = args.id;
        orderbook.base_mint = args.base_mint;
        orderbook.quote_mint = args.quote_mint;

        Ok(())
    }
}
