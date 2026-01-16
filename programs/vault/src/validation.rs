use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::errors::*;

/// Validate token account is properly initialized and not frozen
pub fn validate_token_account(
    token_account: &Account<TokenAccount>,
    expected_owner: &Pubkey,
    expected_mint: &Pubkey,
) -> Result<()> {
    // Check owner
    require!(
        token_account.owner == *expected_owner,
        VaultError::InvalidTokenAccountOwner
    );

    // Check mint
    require!(
        token_account.mint == *expected_mint,
        VaultError::InvalidTokenMint
    );

    // Check not frozen (would be checked by Token program, but explicit check for clarity)
    require!(
        token_account.state == anchor_spl::token::spl_token::state::AccountState::Initialized,
        VaultError::TokenAccountNotInitialized
    );

    Ok(())
}

/// Validate bet amount is within allowed range
pub fn validate_bet_amount(amount: u64) -> Result<()> {
    require!(
        amount >= crate::state::MIN_BET_LAMPORTS && amount <= crate::state::MAX_BET_LAMPORTS,
        VaultError::InvalidBetAmount
    );
    Ok(())
}

/// Validate allowance parameters
pub fn validate_allowance_params(amount: u64, duration_seconds: i64) -> Result<()> {
    require!(
        duration_seconds > 0 && duration_seconds <= crate::state::MAX_ALLOWANCE_DURATION,
        VaultError::AllowanceDurationTooLong
    );

    require!(
        amount <= crate::state::MAX_ALLOWANCE_AMOUNT,
        VaultError::AllowanceAmountTooHigh
    );

    Ok(())
}

/// Validate bet ID format
pub fn validate_bet_id(bet_id: &str) -> Result<()> {
    require!(
        !bet_id.is_empty() && bet_id.len() <= crate::state::ProcessedBet::MAX_BET_ID_LEN,
        VaultError::InvalidBetId
    );
    Ok(())
}

/// Checked arithmetic operations
pub trait CheckedMath {
    fn safe_add(&self, other: Self) -> Result<Self>
    where
        Self: Sized;
    fn safe_sub(&self, other: Self) -> Result<Self>
    where
        Self: Sized;
    fn safe_mul(&self, other: Self) -> Result<Self>
    where
        Self: Sized;
}

impl CheckedMath for u64 {
    fn safe_add(&self, other: Self) -> Result<Self> {
        self.checked_add(other)
            .ok_or_else(|| error!(VaultError::ArithmeticOverflow))
    }

    fn safe_sub(&self, other: Self) -> Result<Self> {
        self.checked_sub(other)
            .ok_or_else(|| error!(VaultError::ArithmeticUnderflow))
    }

    fn safe_mul(&self, other: Self) -> Result<Self> {
        self.checked_mul(other)
            .ok_or_else(|| error!(VaultError::ArithmeticOverflow))
    }
}
