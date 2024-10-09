use crate::{utils::perform_basic_checks, Config, Deposit};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Deposit {
        amount,
        max_x,
        max_y,
        expiration,
    } = Deposit::try_from(data)?;

    let [user, mint_x, mint_y, mint_lp, user_x, user_y, user_lp, vault_x, vault_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure user is signer
    assert!(user.is_signer);

    // Ensure correct TokenProgram
    assert_eq!(token_program.key, &spl_token::ID);

    // Load Config
    let config_account = Config::try_from(config.data.borrow().as_ref())?;

    // Perform Basic Checks
    perform_basic_checks( &config_account, expiration, config, mint_lp, vault_x, vault_y)?;
    
    // Perform User Deposit
    Config::perform_user_deposit(amount, max_x, max_y, &config_account, token_program.key, user_x,
        user_y, user_lp, vault_x, vault_y, mint_x, mint_y, mint_lp, config, user)
}



