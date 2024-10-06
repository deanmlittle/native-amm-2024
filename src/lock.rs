use crate::Withdraw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {

    let [user, mint_x, mint_y, mint_lp, vault_x, vault_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Ok(())
}
