use crate::constants::ORDERBOOK_PDA_SEED;
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DelegateOrderbookArgs {
    pub id: Pubkey,
}

#[delegate]
#[derive(Accounts)]
#[instruction(args: DelegateOrderbookArgs)]
pub struct DelegateOrderbook<'info> {
    pub payer: Signer<'info>,
    /// CHECK: The orderbook pda
    #[account(mut, del)]
    pub pda: AccountInfo<'info>,
}

impl<'info> DelegateOrderbook<'info> {
    pub fn handler(ctx: Context<Self>, args: DelegateOrderbookArgs) -> Result<()> {
        ctx.accounts.delegate_pda(
            &ctx.accounts.payer,
            &[ORDERBOOK_PDA_SEED, args.id.as_ref()],
            DelegateConfig::default(),
        )?;

        Ok(())
    }
}
