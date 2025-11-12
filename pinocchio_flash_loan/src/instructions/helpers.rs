use pinocchio::program_error::ProgramError;

#[repr(C, packed)]
pub struct LoanData {
    pub protocol_token_account: [u8; 32],
    pub balance: u64,
}

pub fn get_token_amount(data: &[u8]) -> Result<u64, ProgramError> {
    if !account.is_owned_by(&pinocchio_token::ID) {
        return Err(PinocchioError::InvalidOwner.into());
    }

    if account
        .data_len()
        .ne(&pinocchio_token::state::TokenAccount::LEN)
    {
        return Err(PinocchioError::InvalidAccountData.into());
    }

    Ok(u64::from_le_bytes(data[64..72].try_into().unwrap()))
}
