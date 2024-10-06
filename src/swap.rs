use std::io::Read;

use crate::{
    utils::{
        check_eq_program_derived_address_and_get_bump, check_eq_program_derived_address_with_bump,
    },
    Config, Deposit, Swap,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, program_pack::Pack, sysvar::Sysvar,
};
use spl_token::state::{GenericTokenAccount, Mint};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Swap {
        amount, // Amount of tokens we deposit
        min,    // Minimum amount of tokens I'd be willing to withdraw
        expiration,
    } = Swap::try_from(data)?;

    assert!(Clock::get()?.unix_timestamp > expiration);

    let [user, mint_x, mint_y, user_x, user_y, vault_x, vault_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure user is signer
    assert!(user.is_signer);

    // Ensure correct TokenProgram
    assert_eq!(token_program.key, &spl_token::ID);

    // Assert we own config
    assert_eq!(config.owner, &crate::ID);
    let config_account = Config::try_from(config.data.borrow().as_ref())?;

    // Check vault accounts are correct
    check_eq_program_derived_address_with_bump(
        &[
            b"vault",
            config_account.mint_x.as_ref(),
            config.key.as_ref(),
            &[config_account.x_bump],
        ],
        &crate::ID,
        vault_x.key,
    )?;

    check_eq_program_derived_address_with_bump(
        &[
            b"vault",
            config_account.mint_y.as_ref(),
            config.key.as_ref(),
            &[config_account.y_bump],
        ],
        &crate::ID,
        vault_y.key,
    )?;

    // Calculate the amount from the amount in the vault:

    let amount_x_in = 1337;
    let amount_y_out = 1337;

    // Slippage check
    assert!(amount_y_out >= min);

    let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
    let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;

    Ok(())
}
