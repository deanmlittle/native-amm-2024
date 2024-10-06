use crate::{utils::check_eq_program_derived_address_with_bump, Config, Swap};
use constant_product_curve::{
    delta_x_from_y_swap_amount_with_fee, delta_y_from_x_swap_amount_with_fee,
};
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::{
    instruction::transfer_checked,
    state::{GenericTokenAccount, Mint},
};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Swap {
        amount,     // Amount of tokens we deposit
        min,        // Minimum amount of tokens we're willing to withdraw
        expiration, // Maximum time for white a swap is valid
    } = Swap::try_from(data)?;

    // Expiration check
    assert!(Clock::get()?.unix_timestamp <= expiration);

    let [user, mint_x, mint_y, user_from, user_to, vault_x, vault_y, config, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure user is signer
    assert!(user.is_signer);

    // Assert we are using the correct TokenProgram
    assert_eq!(token_program.key, &spl_token::ID);

    // Assert we own config
    assert_eq!(config.owner, &crate::ID);
    let config_account = Config::try_from(config.data.borrow().as_ref())?;

    // Assert pool isn't locked
    assert_ne!(config_account.locked, 1);

    // Generate our vault seeds
    let seeds_x: &[&[u8]] = &[
        config_account.mint_x.as_ref(),
        config.key.as_ref(),
        &[config_account.x_bump],
    ];

    let seeds_y: &[&[u8]] = &[
        config_account.mint_y.as_ref(),
        config.key.as_ref(),
        &[config_account.y_bump],
    ];

    // If we check our vault accounts are derived correctly, we don't need to check anything else
    check_eq_program_derived_address_with_bump(seeds_x, &crate::ID, vault_x.key)?;
    check_eq_program_derived_address_with_bump(seeds_y, &crate::ID, vault_y.key)?;

    // Unpack our vault accounts
    let vault_x_account = spl_token::state::Account::unpack(vault_x.data.borrow().as_ref())?;
    let vault_y_account = spl_token::state::Account::unpack(vault_y.data.borrow().as_ref())?;

    // Get our mint decimals
    let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
    let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;

    // No need for additional checks as token transfer will fail for an invalid mint
    let is_x = config_account.mint_x.eq(
        <spl_token::state::Account as GenericTokenAccount>::unpack_account_mint(
            user_from.data.borrow().as_ref(),
        )
        .ok_or(ProgramError::InvalidAccountData)?,
    );

    // Execute our swap
    if is_x {
        // Get amount out less fees
        let (amount_out, _) = delta_y_from_x_swap_amount_with_fee(
            vault_x_account.amount,
            vault_y_account.amount,
            amount,
            config_account.fee,
        )
        .map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Slippage check
        assert!(amount_out >= min);

        deposit(
            token_program.key,
            user_from,
            mint_x,
            vault_x,
            user,
            amount,
            mint_x_decimals,
        )?;

        withdraw(
            token_program.key,
            vault_y,
            mint_y,
            user_to,
            config,
            amount_out,
            mint_y_decimals,
            seeds_y,
        )
    } else {
        // Get amount out less fees
        let (amount_out, _) = delta_x_from_y_swap_amount_with_fee(
            vault_x_account.amount,
            vault_y_account.amount,
            amount,
            config_account.fee,
        )
        .map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Slippage check
        assert!(amount_out >= min);

        // Deposit
        deposit(
            token_program.key,
            user_from,
            mint_y,
            vault_y,
            user,
            amount,
            mint_y_decimals,
        )?;
        // Withdraw
        withdraw(
            token_program.key,
            vault_x,
            mint_x,
            user_to,
            config,
            amount_out,
            mint_x_decimals,
            seeds_x,
        )
    }
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
    vault: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    user_to: &AccountInfo<'a>,
    config: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &transfer_checked(
            token_program,
            vault.key,
            mint.key,
            user_to.key,
            config.key,
            &[],
            amount,
            decimals,
        )?,
        &[vault.clone(), mint.clone(), user_to.clone(), config.clone()],
        &[signer_seeds],
    )
}
