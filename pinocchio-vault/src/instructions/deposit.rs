use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
};
use pinocchio_system::ID;

pub struct DepositAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        };

        if !vault.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        };

        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidArgument);
        };

        let (derived_address, _) = find_program_address(&[b"vault", owner.key()], &crate::ID);

        if vault.key().ne(&derived_address) {
            return Err(ProgramError::InvalidAccountOwner);
        };

        Ok(Self { owner, vault })
    }
}

pub struct DepositInstructionData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(amount: &'a [u8]) -> Result<Self, Self::Error> {
        if amount.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        let le_amount = u64::from_le_bytes(amount.try_into().unwrap());

        if le_amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        };

        Ok(Self { amount: le_amount })
    }
}

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_datas: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(value.1).unwrap();
        let instruction_datas = DepositInstructionData::try_from(value.0).unwrap();

        Ok(Self {
            accounts,
            instruction_datas,
        })
    }
}

