use anchor_lang::{prelude::*, system_program};

use crate::errors::VaultError;
mod errors;

declare_id!("99M4ACDBBrqnaBtkVzk5JPhUGbGunbEBTRKUZvGBNVdh");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

// Initialize Vault_state and vault
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
        seeds = [b"state", user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // Set the fields of the VaultState Account
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        // Transfer the lamports needed to initialize the systemAccount and transfer
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.data_len());
        self.vault_state.rent_exempt = rent_exempt;

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = system_program::Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        system_program::transfer(CpiContext::new(cpi_program, cpi_accounts), rent_exempt)
    }
}

// Deposit to vault
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // Transfer the amount user wants to deposit to the vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_account = system_program::Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        system_program::transfer(CpiContext::new(cpi_program, cpi_account), amount)
    }
}

// Withdraw from vault
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        // Verify amount asked is lower than the amount in vault excluding rent exemption
        let vault_amount = self.vault.try_lamports()?;
        require!(
            amount < vault_amount - self.vault_state.rent_exempt,
            VaultError::InsufficientFunds
        );
        // Transfer the amount user wants to withdraw from the vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_account = system_program::Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let signer_seeds: &[&[&[u8]]] = &[&[
            "vault".as_bytes(),
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ]];
        system_program::transfer(
            CpiContext::new(cpi_program, cpi_account).with_signer(signer_seeds),
            amount,
        )
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        close = user,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let amount = self.vault.try_lamports()?;
        let cpi_program = self.system_program.to_account_info();
        let cpi_account = system_program::Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let signer_seeds: &[&[&[u8]]] = &[&[
            "vault".as_bytes(),
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ]];
        system_program::transfer(
            CpiContext::new(cpi_program, cpi_account).with_signer(signer_seeds),
            amount,
        )
    }
}

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    state_bump: u8,
    vault_bump: u8,
    rent_exempt: u64,
}
