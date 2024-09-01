use {
    crate::{constants::FEE, error::EscrowError},
    anchor_lang::prelude::*,
};

pub fn get_fee(amount: u64) -> Result<u64> {
    let platform_fee = amount.checked_div(FEE * 100).ok_or(EscrowError::Overflow)?;
    Ok(platform_fee)
}
