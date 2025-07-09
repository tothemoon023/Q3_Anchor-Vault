#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

declare_id!("5b4qnT6fNUqB49bDzqHjA6NynfPmTd5bu7GAADfCsY4Y");

#[program]
pub mod vault {
    use super::*;

    // Create a new vault for the user
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
      ctx.accounts.initialize(&ctx.bumps)
    }
    // Put money into the vault
    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }
    // Take money out of the vault
       pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
      // Close the vault and get your money back
      pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}



// Accounts needed to create a new vault    
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // The person creating the vault
    #[account(
        init,
        payer = signer,
        seeds = [b"state", signer.key().as_ref()], // Unique ID for this vault
        bump,
        space = 8 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>, // Stores vault info

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()], // The actual money storage
        bump
    )]
    pub vault: SystemAccount<'info>, // This account holds the SOL

    pub system_program: Program<'info, System> // Needed to transfer SOL
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // Calculate how much SOL we need to keep this account alive
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
        
        // Transfer some SOL to pay for the vault account
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_context, rent_exempt)?;
        
        // Save the bump seeds for later use
        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        Ok(())
    }
}

// Accounts needed for deposits and withdrawals
#[derive(Accounts)]
pub struct Payment<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // The person doing the transaction

    #[account(
        mut,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump, // Use the saved bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump // Use the saved bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Payment<'info> {
    // Move SOL from user to vault
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts: Transfer<'_> = Transfer{
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_context, amount)
    }

    // Move SOL from vault back to user
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer{
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info()
        };
        
        // We need to sign for the vault account since it's a PDA
        let seeds = &[
            &b"vault"[..],
            &self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_context, amount)
    }
}

// Accounts needed to close the vault
#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // The vault owner

    #[account(
        mut,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
        close = signer // Give rent back to the signer
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> Close<'info> {
    // Empty the vault and give all money back to owner
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_account = Transfer{
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info()
        };
        
        // Sign for the vault account to move the money
        let seeds = &[
            &b"vault"[..],
            &self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);
        transfer(cpi_ctx, self.vault.get_lamports()) // Send all remaining SOL
    }
}

// This stores the bump seeds we need to access the vault
#[account]
pub struct VaultState {
    pub vault_bump: u8,    // Bump for the vault account
    pub state_bump: u8,    // Bump for this state account
}

impl Space for VaultState {
    const INIT_SPACE: usize = 1 + 1; // 2 bytes for the two bumps
}