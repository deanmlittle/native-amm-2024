use crate::{utils::check_eq_program_derived_address_with_bump, Config, Deposit};
use constant_product_curve::xy_deposit_amounts_from_l;
use solana_program::{
    account_info::AccountInfo, clock::Clock, sysvar::Sysvar, entrypoint::ProgramResult,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, program::{invoke, invoke_signed},
};
use spl_token::{instruction::{mint_to_checked, transfer_checked}, state::Mint};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Deposit {
        amount,
        max_x,
        max_y,
        expiration,
    } = Deposit::try_from(data)?;

    // Expiration check
    assert!(Clock::get()?.unix_timestamp <= expiration);

    let [user, mint_x, mint_y, mint_lp, user_x, user_y, user_lp, vault_x, vault_y, config, token_program, _system_program] =
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

    // Check LP mint
    check_eq_program_derived_address_with_bump(
        &[config.key.as_ref(), &[config_account.lp_bump]],
        &crate::ID,
        mint_lp.key,
    )?;

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

    check_eq_program_derived_address_with_bump(
        &[
            config_account.mint_y.as_ref(),
            config.key.as_ref(),
            &[config_account.y_bump],
        ],
        &crate::ID,
        vault_y.key,
    )?;

    let vault_x_account = spl_token::state::Account::unpack(&vault_x.try_borrow_data()?)?;
    let vault_y_account = spl_token::state::Account::unpack(&vault_y.try_borrow_data()?)?;
    let mint_lp_account = spl_token::state::Mint::unpack(&mint_lp.try_borrow_data()?)?;

    let (x, y) = xy_deposit_amounts_from_l(
        vault_x_account.amount,
        vault_y_account.amount,
        mint_lp_account.supply,
        amount,
        1_000_000_000,
    )
    .map_err(|_| ProgramError::ArithmeticOverflow)?;

    // Slippage check
    assert!(x <= max_x);
    assert!(y <= max_y);

    // Get decimals
    let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
    let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;

    // Transfer the funds from the users's token X account to the vault
    deposit(
        token_program.key,
        user_x,
        mint_x,
        vault_x,
        user,
        x,
        mint_x_decimals,
    )?;

    deposit(
        token_program.key,
        user_y,
        mint_y,
        vault_y,
        user,
        y,
        mint_y_decimals,
    )?;

    // Mint LP tokens
    mint(
        token_program.key,
        mint_lp,
        user_lp,
        config,
        amount,
        mint_lp_account.decimals,
        config_account.lp_bump,
    )
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
pub fn mint<'a>(
    token_program: &Pubkey,
    mint: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    bump: u8,
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
        &[&[authority.key.as_ref(), &[bump]]],
    )
}
