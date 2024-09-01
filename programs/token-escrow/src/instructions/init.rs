use {
    crate::{constants::ESCROW_SEED, state::Escrow},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
    },
};

#[derive(Accounts)]
#[instruction(data: InitFields)]
pub struct Init<'info> {
    /// the `from` account also defines the authority for an instance of the escrow
    #[account(mut)]
    from: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = from,
    )]
    from_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = from,
        seeds = [
            ESCROW_SEED,
            from.key.as_ref(),
            token_a.key().as_ref(),
            data.token_b.as_ref(),
        ],
        bump,
        space = 8 + Escrow::INIT_SPACE,
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        init_if_needed,
        payer = from,
        associated_token::mint = token_a,
        associated_token::authority = escrow,
    )]
    vault: Account<'info, TokenAccount>,
    token_a: Account<'info, Mint>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitFields {
    to: Pubkey,
    token_b: Pubkey,
    amount_a: u64,
    amount_b: u64,
    expiry: i64,
}

/// Initialises the escrow account and the vault, and transfers `token_a` from `from` into the
/// vault
pub fn process_init(ctx: Context<Init>, data: InitFields) -> Result<()> {
    ctx.accounts.escrow.from = ctx.accounts.from.key();
    ctx.accounts.escrow.to = data.to;
    ctx.accounts.escrow.token_a = ctx.accounts.token_a.key();
    ctx.accounts.escrow.token_b = data.token_b;
    ctx.accounts.escrow.amount_a = data.amount_a;
    ctx.accounts.escrow.amount_b = data.amount_b;
    ctx.accounts.escrow.expiry = data.expiry;

    // Transfer `token_a` from `from` into the vault
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.from_token_account.to_account_info(),
                mint: ctx.accounts.token_a.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.from.to_account_info(),
            },
        ),
        data.amount_a,
        ctx.accounts.token_a.decimals,
    )?;

    Ok(())
}
