use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct RevokeAllowance<'info> {
    #[account(
        mut,
        seeds = [
            b"allowance",
            user.key().as_ref(),
            allowance.casino.as_ref(),
            &allowance.nonce.to_le_bytes()
        ],
        bump = allowance.bump,
        constraint = allowance.user == user.key()
    )]
    pub allowance: Account<'info, Allowance>,

    pub user: Signer<'info>,
}

pub fn handler(ctx: Context<RevokeAllowance>) -> Result<()> {
    let allowance = &mut ctx.accounts.allowance;

    allowance.revoked = true;

    msg!("Allowance revoked for user: {}", ctx.accounts.user.key());

    Ok(())
}
