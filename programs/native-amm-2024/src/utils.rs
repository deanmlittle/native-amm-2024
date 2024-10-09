use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey, 
    account_info::AccountInfo, 
    rent::Rent, 
    entrypoint::ProgramResult, 
    clock::Clock,
    sysvar::Sysvar,     
    program::{invoke, invoke_signed},
    system_instruction::create_account,
    program_pack::Pack, 
};
use spl_token::instruction::{initialize_account3, initialize_mint2, transfer_checked, mint_to_checked, burn_checked};
use crate::state::Config;

#[inline]
pub fn check_eq_program_derived_address_with_bump(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) -> Result<(), ProgramError> {
    let derived_address = Pubkey::create_program_address(seeds, program_id)?;
    Ok(assert!(derived_address.eq(address)))
}

#[inline]
pub fn check_eq_program_derived_address_and_get_bump(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) -> Result<u8, ProgramError> {
    let (derived_address, bump) = Pubkey::try_find_program_address(seeds, program_id)
        .ok_or(ProgramError::InvalidAccountData)?;
    assert!(derived_address.eq(address));
    Ok(bump)
}

#[inline]
pub fn create_token_account<'a>(
    seeds: &[&[u8]],
    token_program: &Pubkey,
    payer: &AccountInfo<'a>,
    ta: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
) -> ProgramResult {
    let token_space = spl_token::state::Account::LEN;
    let token_rent = Rent::get()?.minimum_balance(token_space);

    invoke_signed(
        &create_account(
            payer.key,
            ta.key,
            token_rent,
            token_space as u64,
            &spl_token::ID,    
        ),
        &[payer.clone(), ta.clone()],
        &[seeds],
    )?;

    invoke(
        &initialize_account3(token_program, ta.key, mint.key, authority.key)?,
        &[ta.clone(), mint.clone()],
    )
}

#[inline]
pub fn create_mint<'a>(
    seeds: &[&[u8]],
    token_program: &Pubkey,
    payer: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
) -> ProgramResult {
    let mint_space = spl_token::state::Mint::LEN;
    let mint_rent = Rent::get()?.minimum_balance(mint_space);

    invoke_signed(
        &create_account(
            payer.key,
            mint.key,
            mint_rent,
            mint_space as u64,
            &spl_token::ID,
        ),
        &[payer.clone(), mint.clone()],
        &[seeds],
    )?;

    invoke(
        &initialize_mint2(token_program, mint.key, authority.key, None, 0)?,
        &[mint.clone()],
    )
}

#[inline]
pub fn perform_basic_checks(
    config_account: &Config,
    expiration: i64,
    config: &AccountInfo,
    mint_lp: &AccountInfo,
    vault_x: &AccountInfo,
    vault_y: &AccountInfo,
) -> ProgramResult {
    // Expiration check
    assert!(Clock::get()?.unix_timestamp <= expiration);
    
    // Assert we own config
    assert_eq!(config.owner, &crate::ID);

    // Assert pool isn't locked
    assert_ne!(config_account.locked, 1);

    // Check LP mint
    check_eq_program_derived_address_with_bump(
        &[config.key.as_ref(), &[config_account.lp_bump]],
        &crate::ID,
        mint_lp.key,
    )?;

    // Check vault X
    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_x.as_ref(),
            config.key.as_ref(),
            &[config_account.x_bump],
        ],
        &crate::ID,
        vault_x.key,
    )?;

    // Check vault Y
    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_y.as_ref(),
            config.key.as_ref(),
            &[config_account.y_bump],
        ],
        &crate::ID,
        vault_y.key,
    )?;

    Ok(())
}

pub fn perform_basic_checks_with_no_lp(
    config_account: &Config,
    expiration: i64,
    config: &AccountInfo,
    vault_x: &AccountInfo,
    vault_y: &AccountInfo,
) -> ProgramResult {
    // Expiration check
    assert!(Clock::get()?.unix_timestamp <= expiration);
    
    // Assert we own config
    assert_eq!(config.owner, &crate::ID);

    // Assert pool isn't locked
    assert_ne!(config_account.locked, 1);

    // Check vault X
    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_x.as_ref(),
            config.key.as_ref(),
            &[config_account.x_bump],
        ],
        &crate::ID,
        vault_x.key,
    )?;

    // Check vault Y
    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_y.as_ref(),
            config.key.as_ref(),
            &[config_account.y_bump],
        ],
        &crate::ID,
        vault_y.key,
    )?;

    Ok(())
}

#[inline]
pub fn deposit<'a>(
    token_program: &Pubkey,
    user_from: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    vault: &AccountInfo<'a>,
    user: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
) -> ProgramResult {
    // Transfer the funds from the maker's token account to the vault
    invoke(
        &transfer_checked(
            token_program,
            user_from.key,
            mint.key,
            vault.key,
            user.key,
            &[],
            amount,
            decimals,
        )?,
        &[user_from.clone(), mint.clone(), vault.clone(), user.clone()],
    )
}

#[inline]
pub fn withdraw<'a>(
    token_program: &Pubkey,
    user_to: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    vault: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    seeds: &[&[u8]],
) -> ProgramResult {
    // Transfer the funds from the maker's token account to the vault
    invoke_signed(
        &transfer_checked(
            token_program,
            vault.key,
            mint.key,
            user_to.key,
            authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[
            user_to.clone(),
            mint.clone(),
            vault.clone(),
            authority.clone(),
        ],
        &[seeds],
    )
}

#[inline]
pub fn mint<'a>(
    token_program: &Pubkey,
    mint: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    seeds: &[&[u8]],
) -> ProgramResult {
    // Transfer the funds from the maker's token account to the vault
    invoke_signed(
        &mint_to_checked(
            token_program,
            mint.key,
            to.key,
            authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[mint.clone(), to.clone(), authority.clone()],
        &[seeds],
    )
}

#[inline]
pub fn burn<'a>(
    token_program: &Pubkey,
    from: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    user: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
) -> ProgramResult {
    // Burn the funds from the user's token account
    invoke(
        &burn_checked(
            token_program,
            from.key,
            mint.key,
            user.key,
            &[],
            amount,
            decimals,
        )?,
        &[from.clone(), mint.clone(), user.clone()],
    )
}

#[inline]
pub fn execute_swap<'a>(
    token_program_key: &Pubkey,
    amount: u64,
    amount_out: u64,
    config_account: &Config,
    decimals_from: u8,
    decimals_to: u8,
    config: &AccountInfo<'a>,
    user_from: &AccountInfo<'a>,
    user_to: &AccountInfo<'a>,
    mint_from: &AccountInfo<'a>,
    mint_to: &AccountInfo<'a>,
    vault_from: &AccountInfo<'a>,
    vault_to: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    // Deposit the token from the user
    deposit(
        token_program_key,
        user_from,
        mint_from,
        vault_from,
        user_from,
        amount,
        decimals_from,
    )?;

    // Withdraw the corresponding token to the user
    withdraw(
        token_program_key,
        vault_to,
        mint_to,
        user_to,
        config,
        amount_out,
        decimals_to,
        &[b"config", config_account.seed.to_le_bytes().as_ref(), &[config_account.config_bump]],
    )?;

    Ok(())
}