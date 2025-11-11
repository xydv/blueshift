use anchor_lang::prelude::*;
use anchor_lang::{
    solana_program::sysvar::instructions::{
        load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
    },
    Discriminator,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_flash_loan {
    use anchor_lang::prelude::sysvar::instructions::load_current_index_checked;

    use super::*;

    pub fn borrow(ctx: Context<Loan>, borrow_amount: u64) -> Result<()> {
        require!(borrow_amount > 0, ProtocolError::InvalidAmount);

        let signer_seeds: &[&[&[u8]]] = &[&[b"protocol".as_ref(), &[ctx.bumps.protocol]]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.protocol_ata.to_account_info(),
                    to: ctx.accounts.borrower_ata.to_account_info(),
                    authority: ctx.accounts.protocol.to_account_info(),
                },
                signer_seeds,
            ),
            borrow_amount,
        )?;

        let ixs = ctx.accounts.instructions.to_account_info();

        let index = load_current_index_checked(&ixs)?;

        // check first ix is borrow ix
        require_eq!(index, 0, ProtocolError::InvalidInstructionIndex);

        let data = ixs.try_borrow_data()?;
        let length = u16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;

        match load_instruction_at_checked(length - 1, &ixs) {
            Ok(ix) => {
                require_keys_eq!(ix.program_id, crate::ID, ProtocolError::InvalidProgram);

                // discriminator
                require!(
                    ix.data[0..8].eq(crate::instruction::Repay::DISCRIMINATOR),
                    ProtocolError::InvalidAmount
                );

                require_keys_eq!(
                    ix.accounts
                        .get(3)
                        .ok_or(ProtocolError::InvalidBorrowerAta)?
                        .pubkey,
                    ctx.accounts.borrower_ata.key(),
                    ProtocolError::InvalidBorrowerAta
                );

                require_keys_eq!(
                    ix.accounts
                        .get(4)
                        .ok_or(ProtocolError::InvalidProtocolAta)?
                        .pubkey,
                    ctx.accounts.protocol_ata.key(),
                    ProtocolError::InvalidProtocolAta
                );
            }
            Err(_) => return err!(ProtocolError::MissingRepayIx),
        }

        Ok(())
    }

    pub fn repay(ctx: Context<Loan>) -> Result<()> {
        let ixs = ctx.accounts.instructions.to_account_info();
        let mut amount_borrowed: u64;

        match load_instruction_at_checked(0, &ixs) {
            Ok(ix) => amount_borrowed = u64::from_le_bytes(ix.data[8..16].try_into().unwrap()),
            Err(_) => return err!(ProtocolError::MissingBorrowIx),
        }

        let fee = (amount_borrowed as u128)
            .checked_mul(500)
            .unwrap()
            .checked_div(10_000)
            .ok_or(ProtocolError::Overflow)? as u64;

        amount_borrowed = amount_borrowed
            .checked_add(fee)
            .ok_or(ProtocolError::Overflow)?;

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.borrower_ata.to_account_info(),
                    to: ctx.accounts.protocol_ata.to_account_info(),
                    authority: ctx.accounts.borrower.to_account_info(),
                },
            ),
            amount_borrowed,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Loan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: SystemAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower,
    )]
    pub borrower_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol,
    )]
    pub protocol_ata: Account<'info, TokenAccount>,

    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    /// CHECK: InstructionsSysvar account
    pub instructions: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ProtocolError {
    #[msg("Invalid instruction")]
    InvalidIx,
    #[msg("Invalid instruction index")]
    InvalidInstructionIndex,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Not enough funds")]
    NotEnoughFunds,
    #[msg("Program Mismatch")]
    ProgramMismatch,
    #[msg("Invalid program")]
    InvalidProgram,
    #[msg("Invalid borrower ATA")]
    InvalidBorrowerAta,
    #[msg("Invalid protocol ATA")]
    InvalidProtocolAta,
    #[msg("Missing repay instruction")]
    MissingRepayIx,
    #[msg("Missing borrow instruction")]
    MissingBorrowIx,
    #[msg("Overflow")]
    Overflow,
}
