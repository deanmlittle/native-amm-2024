use crate::{utils::check_eq_program_derived_address_and_get_bump, Config, Initialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    rent::Rent,
    program::{invoke, invoke_signed},
    system_instruction::create_account,
    sysvar::Sysvar,
    msg
};
// use solana_invoke::{invoke, invoke_signed};
use spl_token::solana_program::program_pack::Pack;
use spl_token::instruction::{initialize_account3, initialize_mint2};

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
    let config_bump = check_eq_program_derived_address_and_get_bump(
        &[b"config", seed.to_le_bytes().as_ref()],
        &crate::ID,
        config.key,
    )?;

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

    // Check that the fee is less than 100%
    assert!(fee < 10_000);

    // Check Mint and Token Program are valid
    let _ = spl_token::state::Mint::unpack(&mint_x.try_borrow_data()?);
    let _ = spl_token::state::Mint::unpack(&mint_y.try_borrow_data()?);

    assert_eq!(spl_token::ID, *token_program.key);

    // Initialize the Config Account
    let config_space = core::mem::size_of::<Config>();
    let config_rent = Rent::get()?.minimum_balance(config_space);

    msg!("Got here");

    // Create the Config Account
    invoke_signed(
        &create_account(
            initializer.key,
            config.key,
            config_rent,
            config_space as u64,
            &crate::ID,
        ),
        &[initializer.clone(), config.clone()],
        &[&[b"config", seed.to_le_bytes().as_ref(), &[config_bump]]],
    )?;

    config.assign(&crate::ID);

    let mut config_data: Config =
        *bytemuck::try_from_bytes_mut::<Config>(*config.data.borrow_mut())
            .map_err(|_| ProgramError::InvalidAccountData)?;
    config_data.clone_from(&Config {
        seed,
        authority,
        mint_x: *mint_x.key,
        mint_y: *mint_y.key,
        fee,
        locked: 0,
        config_bump,
        lp_bump,
        x_bump,
        y_bump,
        padding: [0; 1],
    });

    // Create the token_account_x
    let token_space = core::mem::size_of::<spl_token::state::Account>();
    let token_rent = Rent::get()?.minimum_balance(token_space);

    // invoke_signed(
    //     &create_account(
    //         initializer.key,
    //         vault_x.key,
    //         token_rent,
    //         token_space as u64,
    //         &crate::ID,
    //     ),
    //     &[initializer.clone(), vault_x.clone()],
    //     &[&[mint_x.key.as_ref(), config.key.as_ref(), &[x_bump]]],
    // )?;

    // invoke(
    //     &initialize_account3(token_program.key, vault_x.key, mint_x.key, config.key)?,
    //     &[vault_x.clone(), mint_x.clone()],
    // )?;

    // Create the token_account_y
    // invoke_signed(
    //     &create_account(
    //         initializer.key,
    //         vault_y.key,
    //         token_rent,
    //         token_space as u64,
    //         &crate::ID,
    //     ),
    //     &[initializer.clone(), vault_y.clone()],
    //     &[&[mint_y.key.as_ref(), config.key.as_ref(), &[y_bump]]],
    // )?;

    // invoke(
    //     &initialize_account3(token_program.key, vault_y.key, mint_y.key, config.key)?,
    //     &[vault_y.clone(), mint_y.clone()],
    // )?;

    // Create the lp_mint
    let mint_space = core::mem::size_of::<spl_token::state::Mint>();
    let mint_rent = Rent::get()?.minimum_balance(config_space);

    invoke_signed(
        &create_account(
            initializer.key,
            mint_lp.key,
            mint_rent,
            mint_space as u64,
            &crate::ID,
        ),
        &[initializer.clone(), mint_lp.clone()],
        &[&[config.key.as_ref(), &[lp_bump]]],
    )?;

    invoke(
        &initialize_mint2(token_program.key, mint_lp.key, config.key, None, 6)?,
        &[mint_lp.clone()],
    )
}
