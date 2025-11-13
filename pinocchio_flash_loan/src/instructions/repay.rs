use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{get_token_amount, LoanData};

pub struct RepayAccounts<'a> {
    pub borrower: &'a AccountInfo,
    pub loan: &'a AccountInfo,
    pub token_accounts: &'a [AccountInfo],
}

impl<'a> TryFrom<&'a [AccountInfo]> for RepayAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [borrower, loan, token_accounts @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            borrower,
            loan,
            token_accounts,
        })
    }
}

pub struct Repay<'a> {
    pub accounts: RepayAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Repay<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = RepayAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> Repay<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let loan_data = self.accounts.loan.try_borrow_data()?;
        let num_loans = loan_data.len() / size_of::<LoanData>();

        if num_loans.ne(&self.accounts.token_accounts.len()) {
            return Err(ProgramError::InvalidAccountData);
        };

        // validate token account and balance
        for i in 0..num_loans {
            let loan_data_token_account =
                unsafe { *(loan_data.as_ptr().add(i * size_of::<LoanData>()) as *const [u8; 32]) };
            let loan_data_balance = unsafe {
                *(loan_data
                    .as_ptr()
                    .add(i * size_of::<LoanData>() + size_of::<[u8; 32]>())
                    as *const u64)
            };

            if loan_data_token_account != *self.accounts.token_accounts[i].key() {
                return Err(ProgramError::InvalidAccountData);
            };

            let balance = get_token_amount(&self.accounts.token_accounts[i].try_borrow_data()?)?;

            if balance < loan_data_balance {
                return Err(ProgramError::InvalidAccountData);
            };
        }

        drop(loan_data);

        unsafe {
            *self.accounts.borrower.borrow_mut_lamports_unchecked() +=
                *self.accounts.loan.borrow_lamports_unchecked();
            self.accounts.loan.close_unchecked();
        }

        Ok(())
    }
}
