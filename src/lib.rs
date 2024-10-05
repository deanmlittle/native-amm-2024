mod instructions;
use instructions::*;

mod state;
use state::*;

#[cfg(test)]
mod tests;

mod initialize;
// mod deposit;
// mod withdraw;
// mod swap;
// mod lock;

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

    match EscrowInstructions::try_from(discriminator)? {
        EscrowInstructions::Initialize => make::process(accounts, data),
        EscrowInstructions::Deposit => take::process(accounts),
        EscrowInstructions::Withdraw => refund::process(accounts),
        EscrowInstructions::Swap => swap::process(accounts, data),
        EscrowInstructions::Lock => lock::process(accounts, data),
    }
}
