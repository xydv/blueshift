use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::{errors::EscrowError, state::Escrow};

#[derive(Accounts)]
pub struct Take<'info> {
    // other accounts
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(
        mut,
        close = maker, // send rent back to maker
        seeds = ["escrow".as_bytes(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        // check keys with account state
        has_one = maker @ EscrowError::InvalidMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB,
    )]
    pub escrow: Account<'info, Escrow>,

    // token accounts
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_token_b: InterfaceAccount<'info, TokenAccount>,

    // programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    pub fn transfer_to_maker(&self) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata_token_b.to_account_info(),
                    to: self.maker_ata_token_b.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.mint_b.to_account_info(),
                },
            ),
            self.escrow.recieve,
            self.mint_b.decimals,
        )?;

        Ok(())
    }

    pub fn transfer_to_taker_and_close_vault(&self) -> Result<()> {
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.taker_ata_token_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                },
                &[&[
                    "escrow".as_bytes(),
                    self.maker.key().as_ref(),
                    self.escrow.seed.to_le_bytes().as_ref(),
                    &[self.escrow.bump],
                ]],
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;

        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.maker.to_account_info(),
                authority: self.escrow.to_account_info(),
            },
            &[&[
                "escrow".as_bytes(),
                self.maker.key().as_ref(),
                self.escrow.seed.to_le_bytes().as_ref(),
                &[self.escrow.bump],
            ]],
        ))?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Take>) -> Result<()> {
    ctx.accounts.transfer_to_maker()?;
    ctx.accounts.transfer_to_taker_and_close_vault()?;

    Ok(())
}
