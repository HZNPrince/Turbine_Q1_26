use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn_checked, transfer_checked, BurnChecked, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};
use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds =[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program
    )]
    pub user_lp: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_x,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program
    )]
    pub user_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_y,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program
    )]
    pub user_y: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount: u64, // Amount of LP tokens that the user wants to "burn"
        min_x: u64,  // Minimum amount of token X that the user wants to receive
        min_y: u64,  // Minimum amount of token Y that the user wants to receive
    ) -> Result<()> {
        require!(amount != 0, AmmError::ZeroBalance);
        require!(amount <= self.user_lp.amount, AmmError::InsufficientBalance);

        // Calculate the tokens x and y required by the amount
        // formula: token_x = lp_tokens/ total_supply * total_token_x_vault
        let vault_x_amount = self.vault_x.amount;
        let vault_y_amount = self.vault_y.amount;
        let lp_supply = self.mint_lp.supply;

        // required token_x to mint
        let token_x = amount
            .checked_mul(vault_x_amount)
            .ok_or(AmmError::Overflow)?
            .checked_div(lp_supply)
            .ok_or(AmmError::Underflow)?;

        // required token_y to mint
        let token_y = amount
            .checked_mul(vault_y_amount)
            .ok_or(AmmError::Overflow)?
            .checked_div(lp_supply)
            .ok_or(AmmError::Underflow)?;

        // keep account for slippage error
        require!(
            token_x >= min_x && token_y >= min_y,
            AmmError::SlippageExceeded
        );

        self.withdraw_tokens(true, token_x)?;
        self.withdraw_tokens(false, token_y)?;

        self.burn_lp_tokens(amount)?;
        Ok(())
    }
    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        // match the transfer accounts
        let (from, to, mint, mint_decimals) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let transfer_account = TransferChecked {
            from: from,
            mint: mint,
            to: to,
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
            mint_decimals,
        )?;

        Ok(())
    }
    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let burn_accounts = BurnChecked {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.withdrawer.to_account_info(),
        };

        burn_checked(
            CpiContext::new(self.token_program.to_account_info(), burn_accounts),
            amount,
            self.mint_lp.decimals,
        );

        Ok(())
    }
}
