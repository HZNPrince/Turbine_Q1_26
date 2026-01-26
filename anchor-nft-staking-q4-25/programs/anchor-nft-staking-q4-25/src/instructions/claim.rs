use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::errors::StakeError;
use crate::state::{StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"rewards".as_ref(), config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds= [b"config".as_ref()],
        bump,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        require!(self.user_account.points > 0, StakeError::NoPoints);

        let amount = self.user_account.points as f64 * 10e6;
        let mint_to_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.user_reward_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let signer_seeds: &[&[&[u8]]] = &[&[b"config".as_ref(), &[self.config.bump]]];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                mint_to_accounts,
                signer_seeds,
            ),
            amount as u64,
        )?;

        self.user_account.points = 0;

        Ok(())
    }
}
