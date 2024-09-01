use {
    anchor_lang::prelude::*,
    anchor_spl::token::{TransferChecked, transfer_checked, TokenAccount, Mint, Token},
    crate::{state::Escrow, error::EscrowError, constants::ESCROW_SEED},
};

#[derive(Accounts)]
#[instruction(token_b: Pubkey)]
pub struct Cancel<'info> {
    #[account(mut, address = escrow.from @ EscrowError::Unauthorised)]
    authority: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = authority,
    )]
    authority_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [ESCROW_SEED, authority.key().as_ref(), token_a.key().as_ref(), token_b.as_ref()],
        bump,
        close = authority,
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = escrow,
        close = authority,
    )]
    vault: Account<'info, TokenAccount>,
    #[account(address = escrow.token_a @ EscrowError::IncorrectMint)]
    token_a: Account<'info, Mint>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

pub fn process_cancel(ctx: Context<Cancel>, token_b: Pubkey) -> Result<()> {
    // Transfer tokens out of the vault
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.token_a.to_account_info(),
                to: ctx.accounts.authority_ata.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&[
                ESCROW_SEED, 
                ctx.accounts.authority.key().as_ref(), 
                ctx.accounts.token_a.key().as_ref(), 
                token_b.as_ref(),
                &[ctx.bumps.escrow],
            ]],
        ),
        ctx.accounts.escrow.amount_a,
        ctx.accounts.token_a.decimals,
    )?;

    Ok(())
}
