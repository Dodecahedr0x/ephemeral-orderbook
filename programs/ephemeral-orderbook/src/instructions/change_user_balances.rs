use crate::{constants::ORDERBOOK_PDA_SEED, errors::OrderbookError, state::Orderbook};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChangeUserBalancesArgs {
    pub amount_base: u64,
    pub amount_quote: u64,
}

#[derive(Accounts)]
pub struct ChangeUserBalances<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ORDERBOOK_PDA_SEED, orderbook.id.as_ref()],
        bump = orderbook.bump,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub orderbook: Account<'info, Orderbook>,
    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        token::mint = base_mint,
        token::authority = user,
    )]
    pub user_base_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        token::mint = quote_mint,
        token::authority = user,
    )]
    pub user_quote_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        token::mint = base_mint,
        token::authority = orderbook,
    )]
    pub orderbook_base_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        token::mint = quote_mint,
        token::authority = orderbook,
    )]
    pub orderbook_quote_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> ChangeUserBalances<'info> {
    pub fn handler(
        ctx: Context<Self>,
        is_deposit: bool,
        args: ChangeUserBalancesArgs,
    ) -> Result<()> {
        let user_balances = ctx.accounts.orderbook.user_balances(ctx.accounts.user.key);

        let Some(user_balances) = user_balances else {
            return err!(OrderbookError::UnknownUser);
        };
        if !is_deposit && args.amount_base > user_balances.base_balance {
            return err!(OrderbookError::NotEnoughBaseTokens);
        }
        if !is_deposit && args.amount_quote > user_balances.quote_balance {
            return err!(OrderbookError::NotEnoughQuoteTokens);
        }

        if is_deposit {
            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_base_token_account.to_account_info(),
                        to: ctx.accounts.orderbook_base_token_account.to_account_info(),
                        mint: ctx.accounts.base_mint.to_account_info(),
                        authority: ctx.accounts.user.to_account_info(),
                    },
                ),
                args.amount_base,
                ctx.accounts.base_mint.decimals,
            )?;

            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_quote_token_account.to_account_info(),
                        to: ctx.accounts.orderbook_quote_token_account.to_account_info(),
                        mint: ctx.accounts.quote_mint.to_account_info(),
                        authority: ctx.accounts.user.to_account_info(),
                    },
                ),
                args.amount_quote,
                ctx.accounts.quote_mint.decimals,
            )?;
        } else {
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
                        from: ctx.accounts.orderbook_base_token_account.to_account_info(),
                        to: ctx.accounts.user_base_token_account.to_account_info(),
                        mint: ctx.accounts.base_mint.to_account_info(),
                        authority: ctx.accounts.orderbook.to_account_info(),
                    },
                    signer,
                ),
                args.amount_base,
                ctx.accounts.base_mint.decimals,
            )?;

            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.orderbook_quote_token_account.to_account_info(),
                        to: ctx.accounts.user_quote_token_account.to_account_info(),
                        mint: ctx.accounts.quote_mint.to_account_info(),
                        authority: ctx.accounts.orderbook.to_account_info(),
                    },
                    signer,
                ),
                args.amount_quote,
                ctx.accounts.quote_mint.decimals,
            )?;
        }

        Ok(())
    }
}
