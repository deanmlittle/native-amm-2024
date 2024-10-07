use crate::{Config, Withdraw};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let [authority, config] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure user is signer
    assert!(authority.is_signer);

    // Assert we own config
    assert_eq!(config.owner, &crate::ID);
    let mut config_account = Config::try_from(config.data.borrow_mut().as_ref())?;

    // Assert signer is the correct authority
    assert_eq!(authority.key, &config_account.authority);

    // Assert status is not set to revoked (2)
    assert_ne!(config_account.locked, 2);

    // Get the first byte of our IX data
    let (state, _) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Ensure state has a valid value
    assert!(*state < 2);

    // Update lock state
    config_account.locked = *state;

    Ok(())
}
