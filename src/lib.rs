mod instructions;
use instructions::*;

mod state;
use spl_token_2022::extension::confidential_transfer::instruction::deposit;
use state::*;

#[cfg(test)]
mod tests;

mod deposit;
mod initialize;
mod lock;
mod swap;
mod utils;
mod withdraw;

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey, pubkey::Pubkey,
};

const ID: Pubkey = pubkey!("2oXupQcZBcNtq5H1SjzdAZ2eKv1AxiE6XbLk4Ancw2bB");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if program_id.ne(&crate::ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (discriminator, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match AMMInstructions::try_from(discriminator)? {
        AMMInstructions::Initialize => initialize::process(accounts, data),
        AMMInstructions::Deposit => deposit::process(accounts, data),
        AMMInstructions::Withdraw => withdraw::process(accounts, data),
        AMMInstructions::Swap => swap::process(accounts, data),
        AMMInstructions::Lock => lock::process(accounts, data),
    }
}
