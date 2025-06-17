use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_secp256r1_instruction::Secp256r1Pubkey;
use pinocchio_system::{instructions::Transfer, ID};

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

        Ok(Self { owner, vault })
    }
}

pub struct DepositInstructionData {
    pub pubkey: Secp256r1Pubkey,
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let ix: Self = unsafe {
            core::mem::transmute(
                TryInto::<[u8; size_of::<DepositInstructionData>()]>::try_into(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            )
        };

        Ok(ix)
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

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        let (vault_key, _bump) = find_program_address(
            &[
                b"vault",
                &self.instruction_datas.pubkey[..1],
                &self.instruction_datas.pubkey[1..33],
            ],
            &crate::ID,
        );

        if vault_key.ne(self.accounts.vault.key()) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_datas.amount,
        }
        .invoke()
    }
}
