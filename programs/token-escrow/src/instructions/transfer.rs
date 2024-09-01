use {
    crate::{state::Escrow, constants::{FEE_VAULT, ESCROW_SEED}, error::EscrowError, utils::get_fee},
    anchor_lang::prelude::*, anchor_spl::{
        token::{Mint, Token, TokenAccount, transfer_checked, TransferChecked},
        associated_token::AssociatedToken,
    },
};

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(
        mut,
        seeds = [
            ESCROW_SEED, 
            recipient.key().as_ref(), 
            token_a.key().as_ref(), 
            token_b.key().as_ref()
        ],
        bump,
        close = recipient
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        mut, 
        associated_token::mint = token_a,
        associated_token::authority = escrow,
        close = recipient,
    )]
    vault: Account<'info, TokenAccount>,
    #[account(address = escrow.token_a @ EscrowError::IncorrectMint)]
    token_a: Account<'info, Mint>,
    #[account(address = escrow.token_b @ EscrowError::IncorrectMint)]
    token_b: Account<'info, Mint>,
    #[account(
        mut,
        address = escrow.to @ EscrowError::IncorrectAddress,
    )]
    payer: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = payer,
    )]
    payer_ata: Account<'info, TokenAccount>,
    /// CHECK: Validated with constraint
    #[account(address = escrow.from @ EscrowError::IncorrectAddress)]
    recipient: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = token_b,
        associated_token::authority = recipient,
    )]
    recipient_ata: Account<'info, TokenAccount>,
    /// CHECK: Validated by constraint
    #[account(mut, address = FEE_VAULT @ EscrowError::IncorrectAddress)]
    fee_vault: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a,
        associated_token::authority = fee_vault,
    )]
    fee_vault_a: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_b,
        associated_token::authority = fee_vault,
    )]
    fee_vault_b: Account<'info, TokenAccount>, 
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_transfer(ctx: Context<Transfer>) -> Result<()> {
    let clock = Clock::get()?;
    require_gte!(ctx.accounts.escrow.expiry, clock.unix_timestamp, EscrowError::Expired);
    // Get fee for `token_a`
    let token_a_fee = get_fee(ctx.accounts.escrow.amount_a)?;
    let token_b_fee = get_fee(ctx.accounts.escrow.amount_b)?;
    // Transfer fees into vaults
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.payer_ata.to_account_info(),
                mint: ctx.accounts.token_b.to_account_info(),
                to: ctx.accounts.fee_vault_b.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        token_b_fee,
        ctx.accounts.token_b.decimals,
    )?;
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.token_a.to_account_info(),
                to: ctx.accounts.fee_vault_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&[
                ESCROW_SEED, 
                ctx.accounts.recipient.key().as_ref(), 
                ctx.accounts.token_a.key().as_ref(), 
                ctx.accounts.token_b.key().as_ref(),
                &[ctx.bumps.escrow],
            ]],
        ),
        token_a_fee,
        ctx.accounts.token_a.decimals,
    )?;
    // First transfer from the payer's, who is `to` in the escrow's state, token account
    // to the recipient, who is `from` in the escrow's state
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.payer_ata.to_account_info(),
                mint: ctx.accounts.token_b.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        ctx.accounts.escrow.amount_b.checked_sub(token_b_fee).ok_or(EscrowError::Overflow)?,
        ctx.accounts.token_b.decimals,
    )?;

    // Then transfer from the vault to the payer
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.token_a.to_account_info(),
                to: ctx.accounts.payer_ata.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&[
                ESCROW_SEED, 
                ctx.accounts.recipient.key().as_ref(), 
                ctx.accounts.escrow.token_a.as_ref(), 
                ctx.accounts.escrow.token_b.as_ref(),
                &[ctx.bumps.escrow],
            ]],

        ),
        ctx.accounts.escrow.amount_a.checked_sub(token_a_fee).ok_or(EscrowError::Overflow)?,
        ctx.accounts.token_a.decimals,
    )?;

    // Anchor should close the vault and the escrow if the preceding four (4) transactions are
    // successful
    Ok(())
}
