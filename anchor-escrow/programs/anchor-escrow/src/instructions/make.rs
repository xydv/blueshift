use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{errors::EscrowError, state::Escrow};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    // other accounts
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
        seeds = ["escrow".as_bytes(), maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    // token accounts
    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    pub fn populate_escrow(&mut self, seed: u64, amount: u64, bump: u8) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_a.key(),
            recieve: amount,
            bump,
        });

        Ok(())
    }

    pub fn deposit(&self, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.maker_ata_token_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.vault.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Make>, seed: u64, recieve: u64, amount: u64) -> Result<()> {
    require!(recieve > 0, EscrowError::InvalidAmount);
    require!(amount > 0, EscrowError::InvalidAmount);

    ctx.accounts
        .populate_escrow(seed, recieve, ctx.bumps.escrow)?;

    ctx.accounts.deposit(amount)?;

    Ok(())
}
