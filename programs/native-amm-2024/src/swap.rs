use crate::{utils::perform_basic_checks_with_no_lp, Config, Swap};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Swap {
        amount,     // Amount of tokens we deposit
        min,        // Minimum amount of tokens we're willing to withdraw
        expiration, // Maximum time for white a swap is valid
    } = Swap::try_from(data)?;

    let [user, mint_x, mint_y, user_from, user_to, vault_x, vault_y, config, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure user is signer
    assert!(user.is_signer);

    // Assert we are using the correct TokenProgram
    assert_eq!(token_program.key, &spl_token::ID);

    // Load our config account
    let config_account = Config::try_from(config.data.borrow().as_ref())?;

    // Perform basic checks
    perform_basic_checks_with_no_lp(&config_account, expiration, config, vault_x, vault_y)?;

    Config::perform_swap(&config_account, token_program.key, amount, min, mint_x, mint_y, vault_x, vault_y, user_from, user_to, config)
}
