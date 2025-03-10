use crate::constants::TRADER_PDA_SEED;
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DelegateTraderArgs {
    pub orderbook: Pubkey,
}

#[delegate]
#[derive(Accounts)]
#[instruction(args: DelegateTraderArgs)]
pub struct DelegateTrader<'info> {
    pub payer: Signer<'info>,
    /// CHECK: The trader pda
    #[account(mut, del)]
    pub pda: AccountInfo<'info>,
}

impl<'info> DelegateTrader<'info> {
    pub fn handler(ctx: Context<Self>, args: DelegateTraderArgs) -> Result<()> {
        ctx.accounts.delegate_pda(
            &ctx.accounts.payer,
            &[
                TRADER_PDA_SEED,
                args.orderbook.as_ref(),
                ctx.accounts.payer.key().as_ref(),
            ],
            DelegateConfig::default(),
        )?;

        Ok(())
    }
}
