use anchor_lang::error_code;

#[error_code]
pub enum VaultError {
    #[msg("Insufficient funds in the vault")]
    InsufficientFunds,
}
