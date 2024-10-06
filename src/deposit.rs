use crate::{Deposit, Swap};
use constant_product_curve::xy_deposit_amounts_from_l;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};
use solana_sdk::{program::invoke_signed, program_pack::Pack};
use spl_token::instruction::{mint_to, mint_to_checked, transfer_checked};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Deposit {
        amount,
        max_x,
        max_y,
        expiration,
    } = Swap::try_from(data)?;

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
        &[
            b"lp",
            config.key.as_ref(),
            &[config_account.lp_bump],
        ],
        &crate::ID,
        lp_mint.key,
    )?;
    
    // Check vaults
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

    let vault_x_account = spl_token::state::Account::unpack(&vault_x.try_borrow_data()?)?;
    let vault_y_account = spl_token::state::Account::unpack(&vault_y.try_borrow_data()?)?;
    let mint_lp_account = spl_token::state::Mint::unpack(&mint_lp.try_borrow_data()?)?;

    let (x, y) = xy_deposit_amounts_from_l(vault_x_account.amount , vault_y_account.amount, mint_lp_account.supply, amount, 1_000_000_000).map_err(|_| ProgramError::ArithmeticOverflow)?;

    // Slippage check
    assert!(x <= max_x);
    assert!(y <= max_y);

    // Get decimals
    let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
    let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;
    let mint_lp_decimals = Mint::unpack(mint_lp.data.borrow().as_ref())?.decimals;

    // Transfer the funds from the users's token X account to the vault
    invoke(
        &transfer_checked(
            token_program,
            user_x.key,
            mint_x.key,
            vault_x.key,
            user.key,
            &[],
            x,
            mint_x_decimals,
        )?,
        &[
            user_x.clone(),
            mint_x.clone(),
            vault_x.clone(),
            user.clone(),
        ],
    )?;

    // Transfer the funds from the users's token Y account to the vault
    invoke(
        &transfer_checked(
            token_program.key,
            user_y.key,
            mint_y.key,
            vault_y.key,
            user.key,
            &[],
            y,
            mint_y_decimals,
        )?,
        &[
            user_y.clone(),
            mint_y.clone(),
            vault_y.clone(),
            user.clone(),
        ],
    )?;

    // Mint LP tokens
    invoke_signed(
        &mint_to_checked(
            token_program.key,
            mint_lp.key, 
            user_lp.key, 
            config.key, 
            &[], 
            amount, 
            mint_lp_decimals
        ),
        &[
            mint_lp.clone(),
            user_lp.clone(),
            config.clone(),
        ],
        &[&[b"lp_mint", config.key.as_ref(), &[config.lp_bump]]]
    )
}
