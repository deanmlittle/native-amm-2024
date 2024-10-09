use crate::{
    utils::{check_eq_program_derived_address_and_get_bump, create_token_account, create_mint}, 
    Config, 
    Initialize
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};

/// Initialize an AMM and seed with initial liquidity
pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Initialize {
        seed,
        fee,
        authority,
        padding,
    } = Initialize::try_from(data)?;

    let [initializer, mint_x, mint_y, mint_lp, vault_x, vault_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Get the bump and check PDAs
    let x_bump = check_eq_program_derived_address_and_get_bump(
        &[mint_x.key.as_ref(), config.key.as_ref()],
        &crate::ID,
        vault_x.key,
    )?;

    let y_bump = check_eq_program_derived_address_and_get_bump(
        &[mint_y.key.as_ref(), config.key.as_ref()],
        &crate::ID,
        vault_y.key,
    )?;

    let lp_bump = check_eq_program_derived_address_and_get_bump(
        &[config.key.as_ref()],
        &crate::ID,
        mint_lp.key,
    )?;

    // Initialize the Config State
    Config::initialize(
        seed,
        authority,
        fee,
        lp_bump,
        x_bump,
        y_bump,
        mint_x,
        mint_y,
        initializer,
        config,
    )?;

    assert_eq!(spl_token::ID, *token_program.key);

    // Create the x_vault
    create_token_account(
        &[mint_x.key.as_ref(), config.key.as_ref(), &[x_bump]],
        token_program.key,
        initializer,
        vault_x,
        mint_x,
        config,
    )?;

    // Create the y_vault
    create_token_account(
        &[mint_y.key.as_ref(), config.key.as_ref(), &[y_bump]],
        token_program.key,
        initializer,
        vault_y,
        mint_y,
        config,
    )?;

    // Create the lp_mint
    create_mint(
        &[config.key.as_ref(), &[lp_bump]], 
        token_program.key,
        initializer,
        mint_lp,
        config,
    )
}
