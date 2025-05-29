use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;

use instructions::*;

declare_id!("DgkdUz3kCjjQKXyn13GWb3YcvEqbqYf9bCUNuNfbCCwa");

#[program]
pub mod anchor_escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, recieve: u64, amount: u64) -> Result<()> {
        instructions::make::handler(ctx, seed, recieve, amount)
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::handler(ctx)
    }

    // #[instruction(discriminator = 2)]
    // pub fn refund(ctx: Context<Refund>) -> Result<()> {
    //     Ok(())
    // }
}
