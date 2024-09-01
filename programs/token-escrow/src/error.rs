use anchor_lang::error_code;

#[error_code]
pub enum EscrowError {
    #[msg("Unauthorised")]
    Unauthorised,
    #[msg("Incorrect mint address")]
    IncorrectMint,
    #[msg("Incorrect account address")]
    IncorrectAddress,
    #[msg("Expiry date has elapsed")]
    Expired,
    #[msg("Fee amount overflow")]
    Overflow,
}
