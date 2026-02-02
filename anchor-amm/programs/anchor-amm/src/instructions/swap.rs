use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub swapper: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = swapper,
        associated_token::token_program = token_program
    )]
    pub user_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = swapper,
        associated_token::mint = mint_y,
        associated_token::authority = swapper,
        associated_token::token_program = token_program,
    )]
    pub user_y: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        // TODO
        // Swap Direction
        let (to_vault_amount, from_vault_amount) = match is_x {
            true => (self.vault_x.amount, self.vault_y.amount),
            false => (self.vault_y.amount, self.vault_x.amount),
        };

        // Deduct the fee from the calculation
        let fee_amount = (amount as u128)
            .checked_mul(self.config.fee as u128)
            .ok_or(AmmError::Overflow)?
            .checked_div(10000)
            .unwrap() as u64;
        let swap_in = (amount as u128)
            .checked_sub(fee_amount as u128)
            .ok_or(AmmError::Underflow)? as u64;

        // Constant product and swap tokens calculate
        // K = X_vault * Y_vault
        // new_X = X + Amount
        // new_Y = K/new_X
        let k = (to_vault_amount as u128)
            .checked_mul(from_vault_amount as u128)
            .ok_or(AmmError::Overflow)?;

        let new_x = (to_vault_amount as u128)
            .checked_add(swap_in as u128)
            .ok_or(AmmError::Overflow)? as u64;

        let new_y = k.checked_div(new_x as u128).ok_or(AmmError::Underflow)? as u64;

        let swap_out = (from_vault_amount)
            .checked_sub(new_y)
            .ok_or(AmmError::Overflow)?;

        // slippage tolerance limit check
        require!(swap_out >= min, AmmError::SlippageExceeded);

        // vault balance check (Note:Should actually have been vault_balance - rent_exempt )
        require!(swap_out <= from_vault_amount, AmmError::InsufficientBalance);

        self.deposit_tokens(is_x, amount)?;

        self.withdraw_tokens(is_x, swap_out)?;

        // method calls
        Ok(())
    }

    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        // TODO
        let (from, to, mint) = match is_x {
            true => (&mut self.user_x, &mut self.vault_x, &self.mint_x),
            false => (&mut self.user_y, &mut self.vault_y, &self.mint_y),
        };

        let transfer_account = TransferChecked {
            from: from.to_account_info(),
            mint: mint.to_account_info(),
            to: to.to_account_info(),
            authority: self.swapper.to_account_info(),
        };
        transfer_checked(
            CpiContext::new(self.token_program.to_account_info(), transfer_account),
            amount,
            mint.decimals,
        )?;
        Ok(())
    }

    pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        //  Transfer

        let (from, to, mint) = match is_x {
            true => (&mut self.vault_y, &mut self.user_y, &self.mint_y),
            false => (&mut self.vault_x, &mut self.user_x, &self.mint_x),
        };

        let transfer_account = TransferChecked {
            from: from.to_account_info(),
            mint: mint.to_account_info(),
            to: to.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let config_seeds = self.config.seed.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"config", config_seeds.as_ref(), &[self.config.config_bump]]];

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_account,
                signer_seeds,
            ),
            amount,
            mint.decimals,
        )?;

        Ok(())
    }
}
