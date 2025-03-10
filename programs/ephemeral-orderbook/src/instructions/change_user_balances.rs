use crate::{
    constants::{ORDERBOOK_PDA_SEED, TRADER_PDA_SEED},
    errors::OrderbookError,
    state::{Orderbook, Trader},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChangeUserBalancesArgs {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct ChangeUserBalances<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
    )]
    pub orderbook: Box<Account<'info, Orderbook>>,
    #[account(
        mut,
        seeds = [TRADER_PDA_SEED, orderbook.key().as_ref(), user.key().as_ref()],
        bump = trader.bump,
        has_one = orderbook,
        has_one = user,
    )]
    pub trader: Box<Account<'info, Trader>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = orderbook,
    )]
    pub orderbook_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ChangeUserBalances<'info> {
    pub fn validate(&self) -> Result<()> {
        if self.mint.key() != self.orderbook.base_mint
            && self.mint.key() != self.orderbook.quote_mint
        {
            return err!(OrderbookError::InvalidMint);
        }

        Ok(())
    }

    pub fn handler(
        ctx: Context<Self>,
        is_deposit: bool,
        args: ChangeUserBalancesArgs,
    ) -> Result<()> {
        let base_mint = ctx.accounts.orderbook.base_mint;
        let quote_mint = ctx.accounts.orderbook.quote_mint;
        let trader = &mut ctx.accounts.trader;

        if is_deposit {
            if ctx.accounts.mint.key() == base_mint {
                trader.base_balance += args.amount;
            } else {
                trader.quote_balance += args.amount;
            }

            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_token_account.to_account_info(),
                        to: ctx.accounts.orderbook_token_account.to_account_info(),
                        mint: ctx.accounts.mint.to_account_info(),
                        authority: ctx.accounts.user.to_account_info(),
                    },
                ),
                args.amount,
                ctx.accounts.mint.decimals,
            )?;
        } else {
            if ctx.accounts.mint.key() == base_mint {
                if args.amount > trader.base_balance {
                    return err!(OrderbookError::NotEnoughBaseTokens);
                } else {
                    trader.base_balance -= args.amount;
                }
            }
            if ctx.accounts.mint.key() == quote_mint {
                if args.amount > trader.quote_balance {
                    return err!(OrderbookError::NotEnoughBaseTokens);
                } else {
                    trader.quote_balance -= args.amount;
                }
            }

            let seeds = &[
                ORDERBOOK_PDA_SEED.as_ref(),
                ctx.accounts.orderbook.id.as_ref(),
                &[ctx.accounts.orderbook.bump],
            ];
            let signer = &[&seeds[..]];
            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.orderbook_token_account.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        mint: ctx.accounts.mint.to_account_info(),
                        authority: ctx.accounts.orderbook.to_account_info(),
                    },
                    signer,
                ),
                args.amount,
                ctx.accounts.mint.decimals,
            )?;
        }

        Ok(())
    }
}
