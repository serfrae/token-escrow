use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub from: Pubkey,
    pub to: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub expiry: i64,
}
