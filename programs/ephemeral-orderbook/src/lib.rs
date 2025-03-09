use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod constants;
mod errors;
mod instructions;
mod state;

use instructions::*;

declare_id!("21hURyc3yhE9StGsuvShDa4j1cbDPoTf7LMP45LX2vdV");

#[ephemeral]
#[program]
pub mod ephemeral_orderbook {
    use super::*;

    /// Initialize the orderbook.
    pub fn initialize_orderbook(
        ctx: Context<InitializeOrderbook>,
        args: InitializeOrderbookArgs,
    ) -> Result<()> {
        InitializeOrderbook::handler(ctx, args)
    }

    /// Create an order in the orderbook.
    pub fn create_order(ctx: Context<CreateOrder>, args: CreateOrderArgs) -> Result<()> {
        CreateOrder::handler(ctx, args)
    }

    /// Create a user of the orderbook.
    pub fn create_user(ctx: Context<CreateUser>) -> Result<()> {
        CreateUser::handler(ctx)
    }

    /// Deposit tokens into the users balances.
    #[access_control(ctx.accounts.validate())]
    pub fn deposit(ctx: Context<ChangeUserBalances>, args: ChangeUserBalancesArgs) -> Result<()> {
        ChangeUserBalances::handler(ctx, true, args)
    }

    /// Deposit tokens into the users balances.
    #[access_control(ctx.accounts.validate())]
    pub fn withdraw(ctx: Context<ChangeUserBalances>, args: ChangeUserBalancesArgs) -> Result<()> {
        ChangeUserBalances::handler(ctx, false, args)
    }

    /// Match a buy and a sell order.
    pub fn match_order(ctx: Context<MatchOrder>, args: MatchOrderArgs) -> Result<()> {
        MatchOrder::handler(ctx, args)
    }

    /// Delegate the account to the delegation program
    pub fn delegate_orderbook(
        ctx: Context<DelegateOrderbook>,
        args: DelegateOrderbookArgs,
    ) -> Result<()> {
        DelegateOrderbook::handler(ctx, args)
    }

    /// Undelegate the account from the delegation program
    pub fn undelegate_orderbook(ctx: Context<UndelegateOrderbook>) -> Result<()> {
        UndelegateOrderbook::handler(ctx)
    }

    // /// Increment the counter + manual commit the account in the ER.
    // pub fn increment_and_commit(ctx: Context<IncrementAndCommit>) -> Result<()> {
    //     let counter = &mut ctx.accounts.counter;
    //     counter.count += 1;
    //     commit_accounts(
    //         &ctx.accounts.payer,
    //         vec![&ctx.accounts.counter.to_account_info()],
    //         &ctx.accounts.magic_context,
    //         &ctx.accounts.magic_program,
    //     )?;
    //     Ok(())
    // }

    // /// Increment the counter + manual commit the account in the ER.
    // pub fn increment_and_undelegate(ctx: Context<IncrementAndCommit>) -> Result<()> {
    //     let counter = &mut ctx.accounts.counter;
    //     counter.count += 1;
    //     // Serialize the Anchor counter account, commit and undelegate
    //     counter.exit(&crate::ID)?;
    //     commit_and_undelegate_accounts(
    //         &ctx.accounts.payer,
    //         vec![&ctx.accounts.counter.to_account_info()],
    //         &ctx.accounts.magic_context,
    //         &ctx.accounts.magic_program,
    //     )?;
    //     Ok(())
    // }
}

// /// Account for the increment instruction.
// #[derive(Accounts)]
// pub struct Increment<'info> {
//     #[account(mut, seeds = [TEST_PDA_SEED], bump)]
//     pub counter: Account<'info, Counter>,
// }

// /// Account for the increment instruction + manual commit.
// #[commit]
// #[derive(Accounts)]
// pub struct IncrementAndCommit<'info> {
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     #[account(mut, seeds = [TEST_PDA_SEED], bump)]
//     pub counter: Account<'info, Counter>,
// }
