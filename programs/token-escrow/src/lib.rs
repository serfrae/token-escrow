use anchor_lang::prelude::*;

declare_id!("BtDRSPLuXPy9zn6yfE5huNYbGq1rMqxKF5Ac5s189bUk");

mod constants;
mod error;
mod instructions;
mod state;
mod utils;

use instructions::*;

#[program]
pub mod token_escrow {
    use super::*;

    pub fn init(ctx: Context<Init>, data: InitFields) -> Result<()> {
        instructions::init::process_init(ctx, data)?;
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>) -> Result<()> {
        instructions::transfer::process_transfer(ctx)?;
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>, token_b: Pubkey) -> Result<()> {
        instructions::cancel::process_cancel(ctx, token_b)?;
        Ok(())
    }
}
