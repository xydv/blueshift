use pinocchio::program_error::ProgramError;

#[repr(C, packed)]
pub struct LoanData {
    pub protocol_token_account: [u8; 32],
    pub balance: u64,
}

pub fn get_token_amount(data: &[u8]) -> Result<u64, ProgramError> {
    let amount = unsafe { *(data.as_ptr().add(64) as *const u64) };
    Ok(amount)
}
