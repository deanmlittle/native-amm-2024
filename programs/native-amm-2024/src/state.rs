use crate::utils::{check_eq_program_derived_address_and_get_bump, deposit, mint, withdraw, burn, execute_swap};
use constant_product_curve::{xy_deposit_amounts_from_l, xy_withdraw_amounts_from_l, delta_x_from_y_swap_amount_with_fee, delta_y_from_x_swap_amount_with_fee};
use bytemuck::{Pod, Zeroable};
use native_amm_macros::TryFromBytes;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::state::{Mint, GenericTokenAccount};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, TryFromBytes)]
pub struct Config {
    pub seed: u64,
    pub authority: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub fee: u16,
    pub locked: u8,
    pub config_bump: u8,
    pub lp_bump: u8,
    pub x_bump: u8,
    pub y_bump: u8,
    pub padding: [u8; 1],
}

impl Config {
    pub fn initialize<'a>(
        seed: u64,
        authority: Pubkey,
        fee: u16,
        lp_bump: u8,
        x_bump: u8,
        y_bump: u8,
        mint_x: &AccountInfo,
        mint_y: &AccountInfo,
        initializer: &AccountInfo<'a>,
        config: &AccountInfo<'a>,
    ) -> ProgramResult {
        let config_bump = check_eq_program_derived_address_and_get_bump(
            &[b"config", seed.to_le_bytes().as_ref()],
            &crate::ID,
            config.key,
        )?;

        // Check that the fee is less than 100%
        assert!(fee < 10_000);

        // Check Mint and Token Program are valid
        let _ = spl_token::state::Mint::unpack(&mint_x.try_borrow_data()?);
        let _ = spl_token::state::Mint::unpack(&mint_y.try_borrow_data()?);

        // Initialize the Config Account
        let config_space = core::mem::size_of::<Config>();
        let config_rent = Rent::get()?.minimum_balance(config_space);

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

        Ok(())
    }

    pub fn perform_user_deposit<'a>(
        amount: u64,
        max_x: u64,
        max_y: u64,
        config_account: &Config,
        token_program: &Pubkey,
        user_x: &AccountInfo<'a>,
        user_y: &AccountInfo<'a>,
        user_lp: &AccountInfo<'a>,
        vault_x: &AccountInfo<'a>,
        vault_y: &AccountInfo<'a>,
        mint_x: &AccountInfo<'a>,
        mint_y: &AccountInfo<'a>,
        mint_lp: &AccountInfo<'a>,
        config: &AccountInfo<'a>,
        user: &AccountInfo<'a>,
    ) -> ProgramResult {
        let vault_x_account = spl_token::state::Account::unpack(&vault_x.try_borrow_data()?)?;
        let vault_y_account = spl_token::state::Account::unpack(&vault_y.try_borrow_data()?)?;
        let mint_lp_account = spl_token::state::Mint::unpack(&mint_lp.try_borrow_data()?)?;

        let (x,y) = match mint_lp_account.supply == 0 && vault_x_account.amount == 0 && vault_y_account.amount == 0 {
            true => (max_x, max_y),
            false => {
                xy_deposit_amounts_from_l(
                    vault_x_account.amount,
                    vault_y_account.amount,
                    mint_lp_account.supply,
                    amount,
                    1_000_000_000,
                )
                .map_err(|_| ProgramError::ArithmeticOverflow)?
            }
        };

        // Slippage check
        assert!(x <= max_x);
        assert!(y <= max_y);

        // Get decimals
        let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
        let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;

        // Transfer the funds from the users's token X account to the vault
        deposit(
            token_program,
            user_x,
            mint_x,
            vault_x,
            user,
            x,
            mint_x_decimals,
        )?;

        // Transfer the funds from the users's token Y account to the vault
        deposit(
            token_program,
            user_y,
            mint_y,
            vault_y,
            user,
            y,
            mint_y_decimals,
        )?;

        // Mint LP tokens
        mint(
            token_program,
            mint_lp,
            user_lp,
            config,
            amount,
            mint_lp_account.decimals,
            &[b"config", config_account.seed.to_le_bytes().as_ref(), &[config_account.config_bump]],
        )
    }

    pub fn perform_user_withdraw<'a>(
        amount: u64,
        min_x: u64,
        min_y: u64,
        config_account: &Config,
        token_program: &Pubkey,
        user_x: &AccountInfo<'a>,
        user_y: &AccountInfo<'a>,
        user_lp: &AccountInfo<'a>,
        vault_x: &AccountInfo<'a>,
        vault_y: &AccountInfo<'a>,
        mint_x: &AccountInfo<'a>,
        mint_y: &AccountInfo<'a>,
        mint_lp: &AccountInfo<'a>,
        config: &AccountInfo<'a>,
        user: &AccountInfo<'a>,
    ) -> ProgramResult {
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

        // Slippage check
        assert!(x >= min_x);
        assert!(y >= min_y);

        // Get decimals
        let mint_x_decimals = Mint::unpack(mint_x.data.borrow().as_ref())?.decimals;
        let mint_y_decimals = Mint::unpack(mint_y.data.borrow().as_ref())?.decimals;

        // Transfer the funds from the users's token X account to the vault
        withdraw(
            token_program,
            user_x,
            mint_x,
            vault_x,
            config,
            x,
            mint_x_decimals,
            &[b"config", config_account.seed.to_le_bytes().as_ref(), &[config_account.config_bump]],
        )?;

        // Transfer the funds from the users's token Y account to the vault
        withdraw(
            token_program,
            user_y,
            mint_y,
            vault_y,
            config,
            y,
            mint_y_decimals,
            &[b"config", config_account.seed.to_le_bytes().as_ref(), &[config_account.config_bump]],

        )?;

        // Mint LP tokens
        burn(
            token_program,
            user_lp,
            mint_lp,
            user,
            amount,
            mint_lp_account.decimals,
        )
    }

    pub fn perform_swap<'a>(
        config_account: &Config,
        token_program: &Pubkey,
        amount: u64,
        min: u64,
        mint_x: &AccountInfo<'a>,
        mint_y: &AccountInfo<'a>, 
        vault_x: &AccountInfo<'a>,
        vault_y: &AccountInfo<'a>,
        user_from: &AccountInfo<'a>,
        user_to: &AccountInfo<'a>,
        config  : &AccountInfo<'a>,
    ) -> ProgramResult {
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

        // Determine swap direction and fee calculation
        let (amount_out, _) = if is_x {
            delta_y_from_x_swap_amount_with_fee(vault_x_account.amount, vault_y_account.amount, amount, config_account.fee)
        } else {
            delta_x_from_y_swap_amount_with_fee(vault_y_account.amount, vault_x_account.amount, amount, config_account.fee)
        }
        .map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Slippage check
        assert!(amount_out >= min);

        // Execute the swap
        if is_x {
            execute_swap(
                token_program,
                amount,
                amount_out,
                config_account,
                mint_x_decimals,
                mint_y_decimals,
                config,
                user_from,
                user_to,
                mint_x,
                mint_y,
                vault_x,
                vault_y,
            )
        } else {
            execute_swap(
                token_program,
                amount,
                amount_out,
                config_account,
                mint_x_decimals,
                mint_y_decimals,
                config,
                user_from,
                user_to,
                mint_y,
                mint_x,
                vault_y,
                vault_x,
            )
        }
    }
}
