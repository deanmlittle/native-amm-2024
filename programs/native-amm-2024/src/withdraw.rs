use crate::{utils::check_eq_program_derived_address_with_bump, Config, Withdraw};
use constant_product_curve::xy_withdraw_amounts_from_l;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
    msg
};
use spl_token::{
    instruction::{burn_checked, transfer_checked},
    state::Mint,
};
pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Withdraw {
        amount,
        min_x,
        min_y,
        expiration,
    } = Withdraw::try_from(data)?;

    // Expiration check
    assert!(Clock::get()?.unix_timestamp <= expiration);

    let [user, mint_x, mint_y, mint_lp, user_x, user_y, user_lp, vault_x, vault_y, config, token_program] =
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

    // Assert pool isn't locked
    assert_ne!(config_account.locked, 1);

    // Check LP mint
    check_eq_program_derived_address_with_bump(
        &[config.key.as_ref(), &[config_account.lp_bump]],
        &crate::ID,
        mint_lp.key,
    )?;

    msg!("Checked LP mint");

    // Check vaults
    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_x.as_ref(),
            config.key.as_ref(),
            &[config_account.x_bump],
        ],
        &crate::ID,
        vault_x.key,
    )?;

    msg!("Checked vault X");

    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_y.as_ref(),
            config.key.as_ref(),
            &[config_account.y_bump],
        ],
        &crate::ID,
        vault_y.key,
    )?;

    msg!("Checked vault Y");

    let vault_x_account = spl_token::state::Account::unpack(&vault_x.try_borrow_data()?)?;
    let vault_y_account = spl_token::state::Account::unpack(&vault_y.try_borrow_data()?)?;
    let mint_lp_account = spl_token::state::Mint::unpack(&mint_lp.try_borrow_data()?)?;

    let (x, y) = xy_withdraw_amounts_from_l(
        vault_x_account.amount,
        vault_y_account.amount,
        mint_lp_account.supply,
        amount,
        1_000_000_000,
    )
    .map_err(|_| ProgramError::ArithmeticOverflow)?;

    msg!("Calculated withdraw amounts: X: {} Y: {}", x, y);

    msg!("Vault X amount: {}", vault_x_account.amount);
    msg!("Vault Y amount: {}", vault_y_account.amount);

    // Slippage check
    assert!(x >= min_x);
    assert!(y >= min_y);

    // Get decimals
    let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
    let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;

    // Transfer the funds from the users's token X account to the vault
    withdraw(
        token_program.key,
        user_x,
        mint_x,
        vault_x,
        config,
        x,
        mint_x_decimals,
        &[b"config", config_account.seed.to_le_bytes().as_ref(), &[config_account.config_bump]],
    )?;

    msg!("Withdrew X");

    withdraw(
        token_program.key,
        user_y,
        mint_y,
        vault_y,
        config,
        y,
        mint_y_decimals,
        &[b"config", config_account.seed.to_le_bytes().as_ref(), &[config_account.config_bump]],

    )?;

    msg!("Withdrew Y");

    // Mint LP tokens
    burn(
        token_program.key,
        user_lp,
        mint_lp,
        user,
        amount,
        mint_lp_account.decimals,
    )?;

    msg!("Burned LP tokens");

    Ok(())
}

#[inline]
pub fn withdraw<'a>(
    token_program: &Pubkey,
    user: &AccountInfo<'a>,
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
            user.key,
            authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[
            user.clone(),
            mint.clone(),
            vault.clone(),
            authority.clone(),
        ],
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