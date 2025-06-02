#![no_std]

use pinocchio::{
    ProgramResult, account_info::AccountInfo, default_panic_handler, nostd_panic_handler,
    program_entrypoint, pubkey::Pubkey,
};

pub mod instructions;

use instructions::*;

program_entrypoint!(process_instruction);
nostd_panic_handler!();

pub const ID: Pubkey = [
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
];

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // match instruction_data.split_first() {
    //     Some((&0, data)) => {}
    //     None => {}
    // }

    Ok(())
}
