use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_lang::{context::CpiContext, Accounts, Result, ToAccountInfos};
use serum_dex::matching::Side;
use solana_program::entrypoint::ProgramResult;
use std::num::NonZeroU64;

pub use serum_dex;

#[cfg(not(feature = "devnet"))]
anchor_lang::solana_program::declare_id!("9cnJvRQY38Bu7dWUUCncZ53evxZ4mR4S9vYV8BpToh26");

#[cfg(feature = "devnet")]
anchor_lang::solana_program::declare_id!("9cnJvRQY38Bu7dWUUCncZ53evxZ4mR4S9vYV8BpToh26");

#[allow(clippy::too_many_arguments)]
pub fn send_take<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, SendTake<'info>>,
    side: Side,
    limit_price: NonZeroU64,
    max_coin_qty: NonZeroU64,
    max_native_pc_qty_including_fees: NonZeroU64,
    min_coin_qty: u64,
    min_native_pc_qty: u64,
    limit: u16,
) -> ProgramResult {
    let referral = ctx.remaining_accounts.get(0);
    let ix = serum_dex::instruction::send_take(
        ctx.accounts.market.key,
        ctx.accounts.request_queue.key,
        ctx.accounts.event_queue.key,
        ctx.accounts.market_bids.key,
        ctx.accounts.market_asks.key,
        ctx.accounts.coin_wallet.key,
        ctx.accounts.pc_wallet.key,
        ctx.accounts.wallet_owner.key,
        ctx.accounts.coin_vault.key,
        ctx.accounts.pc_vault.key,
        ctx.accounts.token_program.key,
        ctx.accounts.vault_signer.key,
        referral.map(|r| r.key),
        &ID,
        side,
        limit_price,
        max_coin_qty,
        max_native_pc_qty_including_fees,
        min_coin_qty,
        min_native_pc_qty,
        limit,
    )
    .map_err(|pe| ProgramError::from(pe))?;
    solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct SendTake<'info> {
    pub market: AccountInfo<'info>,
    pub request_queue: AccountInfo<'info>,
    pub event_queue: AccountInfo<'info>,
    pub market_bids: AccountInfo<'info>,
    pub market_asks: AccountInfo<'info>,
    pub coin_wallet: AccountInfo<'info>,
    pub pc_wallet: AccountInfo<'info>,
    pub wallet_owner: AccountInfo<'info>,
    pub coin_vault: AccountInfo<'info>,
    pub pc_vault: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub vault_signer: AccountInfo<'info>,
}
