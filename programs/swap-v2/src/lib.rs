//* Program to perform instantly settled token swaps on the Serum DEX.

use std::num::NonZeroU64;

use anchor_lang::prelude::*;
use anchor_spl::token;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod dex;

#[program]
pub mod swap_v2 {
    use super::*;

    /// Convenience API to call the SendTake function on the Serum DEX.
    ///
    /// SendTake does not require an open orders account or settlement of funds for the user -
    /// rather it immediately settles the deposites funds into the user's account if there is a counterparty in the orderbook.
    ///
    /// Thus, this function is useful for instant swaps on a single A/B market,
    /// where A is the base currency and B is the quote currency.
    ///
    /// When side is "bid", then swaps B for A. When side is "ask", then swaps A for B.

    /// * `side`           - The direction to swap.
    /// * `amount`         - The amount to swap "from".
    /// * `amount_out_min` - The minimum amount of the "to" token the client expects to receive from the swap.
    /// The instruction fails if execution would result in less.

    #[access_control(is_valid_swap(&ctx))]
    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
        side: Side,
        amount: u64,
        amount_out_min: u64,
    ) -> Result<()> {
        // Optional referral account (earns a referral fee).
        let srm_msrm_discount = ctx.remaining_accounts.iter().next().map(Clone::clone);
        let orderbook: OrderbookClient<'info> = (&*ctx.accounts).into();

        // Side determines swap direction.
        let (from_token, to_token) = match side {
            Side::Bid => (&ctx.accounts.pc_wallet, &ctx.accounts.market.coin_wallet),
            Side::Ask => (&ctx.accounts.market.coin_wallet, &ctx.accounts.pc_wallet),
        };

        // Calculate the limit price.
        let (
            limit_price,
            max_coin_qty,
            max_native_max_native_pc_qty_including_fees,
            min_coin_qty,
            min_native_pc_qty,
        ) = match side {
            Side::Bid => {
                let limit_price = amount_out_min / amount;
                NonZeroU64::new(limit_price).ok_or(ErrorCode::InvalidLimitPrice)?;
                let max_coin_qty = amount / limit_price;
                let max_native_max_native_pc_qty_including_fees = amount;
                let min_coin_qty = amount_out_min;
                let min_native_pc_qty = amount_out_min * limit_price;
                (
                    limit_price,
                    max_coin_qty,
                    max_native_max_native_pc_qty_including_fees,
                    min_coin_qty,
                    min_native_pc_qty,
                )
            }
            Side::Ask => {
                let limit_price = amount / amount_out_min;
                NonZeroU64::new(limit_price).ok_or(ErrorCode::InvalidLimitPrice)?;
                let max_coin_qty = amount;
                let max_native_max_native_pc_qty_including_fees = amount * limit_price;
                let min_coin_qty = amount_out_min / limit_price;
                let min_native_pc_qty = amount_out_min;
                (
                    limit_price,
                    max_coin_qty,
                    max_native_max_native_pc_qty_including_fees,
                    min_coin_qty,
                    min_native_pc_qty,
                )
            }
        };

        // Token balances before the trade.
        let from_amount_before = token::accessor::amount(from_token)?;
        let to_amount_before = token::accessor::amount(to_token)?;

        orderbook.send_take_cpi(
            side,
            limit_price,
            max_coin_qty,
            max_native_max_native_pc_qty_including_fees,
            min_coin_qty,
            min_native_pc_qty,
            srm_msrm_discount,
        )?;

        // Token balances after the trade.
        let from_amount_after = token::accessor::amount(from_token)?;
        let to_amount_after = token::accessor::amount(to_token)?;

        //  Calculate the delta, i.e. the amount swapped.
        let from_amount = from_amount_before.checked_sub(from_amount_after).unwrap();
        let to_amount = to_amount_after.checked_sub(to_amount_before).unwrap();

        Ok(())
    }

    /// Swap two base currencies across two different markets.
    ///
    /// That is, suppose there are two markets, A/USD(x) and B/USD(x).
    /// Then swaps token A for token B via
    ///
    /// 1. Selling A to USD(x) on A/USD(x) market using SendTake.
    /// 2. Buying B using the proceed USD(x) on B/USD(x) market using SendTake.

    /// * `amount`         - The amount to swap "from".
    /// * `amount_out_min` - The minimum amount of the "to" token the client expects to receive from the swap.
    /// The instruction fails if execution would result in less.

    #[access_control(is_valid_swap_transitive(&ctx))]
    pub fn swap_transitive<'info>(
        ctx: Context<'_, '_, '_, 'info, SwapTransitive<'info>>,
        amount: u64,
        amount_out_min: u64,
    ) -> Result<()> {
        todo!()
    }

    pub fn test<'info>(ctx: Context<TEST>) -> Result<()> {
        msg!("Hello, world!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TEST {}

#[derive(Accounts)]
pub struct Swap<'info> {
    // The single A/B market to swap on
    pub market: MarketAccounts<'info>,
    // The swap user
    #[account(signer)]
    pub wallet_owner: AccountInfo<'info>,
    // The user's token account for the 'price' currency
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    // The Serum DEX program
    pub dex_program: AccountInfo<'info>,
    // The token program
    pub token_program: AccountInfo<'info>,
}

impl<'info> From<&Swap<'info>> for OrderbookClient<'info> {
    fn from(accounts: &Swap<'info>) -> OrderbookClient<'info> {
        OrderbookClient {
            market: accounts.market.clone(),
            wallet_owner: accounts.wallet_owner.clone(),
            pc_wallet: accounts.pc_wallet.clone(),
            dex_program: accounts.dex_program.clone(),
            token_program: accounts.token_program.clone(),
        }
    }
}

#[derive(Accounts)]
pub struct SwapTransitive<'info> {
    // The first A/B market to swap on, A -> B, ask
    pub from: MarketAccounts<'info>,
    // The second C/B market to swap on, B -> C, bid
    pub to: MarketAccounts<'info>,
    // The swap user
    #[account(signer)]
    pub wallet_owner: AccountInfo<'info>,
    // The user's token account for the 'price' currency
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    // The Serum DEX program
    pub dex_program: AccountInfo<'info>,
    // The token program
    pub token_program: AccountInfo<'info>,
}

impl<'info> SwapTransitive<'info> {
    fn orderbook_from(&self) -> OrderbookClient<'info> {
        OrderbookClient {
            market: self.from.clone(),
            wallet_owner: self.wallet_owner.clone(),
            pc_wallet: self.pc_wallet.clone(),
            dex_program: self.dex_program.clone(),
            token_program: self.token_program.clone(),
        }
    }
    fn orderbook_to(&self) -> OrderbookClient<'info> {
        OrderbookClient {
            market: self.to.clone(),
            wallet_owner: self.wallet_owner.clone(),
            pc_wallet: self.pc_wallet.clone(),
            dex_program: self.dex_program.clone(),
            token_program: self.token_program.clone(),
        }
    }
}

// Market accounts are the accounts used to place orders against the dex minus
// common accounts, i.e., program ids, sysvars
#[derive(Accounts, Clone)]
pub struct MarketAccounts<'info> {
    // The DEX markets
    #[account(mut)]
    pub market: AccountInfo<'info>,
    // The DEX request queue
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    // The DEX event queue
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    // The DEX market bids
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    // The DEX market asks
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    // Also known as the "base" currency. For a given A/B market,
    // this is the vault for the A mint.
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    // Also known as the "quote" currency. For a given A/B market,
    // this is the vault for the B mint.
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    // PDA owner of the DEX's token accounts for base + quote currencies.
    #[account(mut)]
    pub vault_signer: AccountInfo<'info>,
    // The user's token account for the 'coin' currency
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
}

// Client for sending orders to the Serum DEX.
#[derive(Clone)]
struct OrderbookClient<'info> {
    market: MarketAccounts<'info>,
    wallet_owner: AccountInfo<'info>,
    pc_wallet: AccountInfo<'info>,
    dex_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

impl<'info> OrderbookClient<'info> {
    /// Execute SendTake on the Serum DEX via CPI.
    fn send_take_cpi(
        &self,
        side: Side,
        limit_price: u64,
        max_coin_qty: u64,
        max_native_max_native_pc_qty_including_fees: u64,
        min_coin_qty: u64,
        min_native_pc_qty: u64,
        srm_msrm_discount: Option<AccountInfo<'info>>,
    ) -> Result<()> {
        let cpi_accounts = dex::SendTake {
            market: self.market.market.clone(),
            request_queue: self.market.request_queue.clone(),
            event_queue: self.market.event_queue.clone(),
            market_bids: self.market.market_bids.clone(),
            market_asks: self.market.market_asks.clone(),
            coin_wallet: self.market.coin_wallet.clone(),
            pc_wallet: self.pc_wallet.clone(),
            wallet_owner: self.wallet_owner.clone(),
            coin_vault: self.market.coin_vault.clone(),
            pc_vault: self.market.pc_vault.clone(),
            token_program: self.token_program.clone(),
            vault_signer: self.market.vault_signer.clone(),
        };
        // Limit is the dex's custom compute budge parameter, setting an upper
        // bound on the number of matching cycles the program can perform
        // before giving up and posting the remaining unmatched order.
        let limit = 65535;
        let mut ctx = CpiContext::new(self.dex_program.clone(), cpi_accounts);
        if let Some(srm_msrm_discount) = srm_msrm_discount {
            ctx = ctx.with_remaining_accounts(vec![srm_msrm_discount]);
        }
        dex::send_take(
            ctx,
            side.into(),
            NonZeroU64::new(limit_price).unwrap(),
            NonZeroU64::new(max_coin_qty).unwrap(),
            NonZeroU64::new(max_native_max_native_pc_qty_including_fees).unwrap(),
            min_coin_qty,
            min_native_pc_qty,
            limit,
        )
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl From<Side> for serum_dex::matching::Side {
    fn from(side: Side) -> Self {
        match side {
            Side::Bid => serum_dex::matching::Side::Bid,
            Side::Ask => serum_dex::matching::Side::Ask,
        }
    }
}

// Access control modifiers.

fn is_valid_swap(ctx: &Context<Swap>) -> Result<()> {
    _is_valid_swap(&ctx.accounts.market.coin_wallet, &ctx.accounts.pc_wallet)
}

fn is_valid_swap_transitive(ctx: &Context<SwapTransitive>) -> Result<()> {
    _is_valid_swap(&ctx.accounts.from.coin_wallet, &ctx.accounts.to.coin_wallet)
}

// Validates the tokens being swapped are of different mints.
fn _is_valid_swap<'info>(from: &AccountInfo<'info>, to: &AccountInfo<'info>) -> Result<()> {
    let from_token_mint = token::accessor::mint(from)?;
    let to_token_mint = token::accessor::mint(to)?;
    if from_token_mint == to_token_mint {
        return Err(ErrorCode::SwapTokensCannotMatch.into());
    }
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("The tokens being swapped must have different mints")]
    SwapTokensCannotMatch,
    #[msg("The implied limit price is invalid")]
    InvalidLimitPrice,
}
