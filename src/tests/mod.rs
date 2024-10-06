use bytemuck::bytes_of;
use core::mem;
use mollusk_svm::{
    program::{self, program_account},
    result::ProgramResult,
    Mollusk,
};
use solana_program::instruction::AccountMeta;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    instruction::Instruction,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::AccountState;

use crate::{Config, Initialize};

#[test]
fn initialize() {
    // Add our built program binary
    let mut mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_amm_2024");

    // Set our seed
    let seed: u64 = 1337;

    // Programs
    mollusk.add_program(&spl_token::ID, "src/tests/spl_token-3.5.0");
    let (token_program, token_program_account) = (spl_token::ID, program_account(&spl_token::ID));
    let (system_program, system_program_account) = program::system_program();

    // Accounts
    let initializer = Pubkey::new_from_array([0x01; 32]);
    let mint_x = Pubkey::new_from_array([0x02; 32]);
    let mint_y = Pubkey::new_from_array([0x03; 32]);
    let config = Pubkey::find_program_address(&[b"config", &seed.to_le_bytes()], &crate::ID).0;
    let mint_lp = Pubkey::find_program_address(&[config.as_ref()], &crate::ID).0;
    let vault_x = Pubkey::find_program_address(&[mint_x.as_ref(), config.as_ref()], &crate::ID).0;
    let vault_y = Pubkey::find_program_address(&[mint_y.as_ref(), config.as_ref()], &crate::ID).0;

    // Fill out our account data
    let mut mint_x_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x05; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_x_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut mint_y_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x06; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_y_account.data_as_mut_slice(),
    )
    .unwrap();

    let vault_x_account = AccountSharedData::new(0, 0, &Pubkey::default());
    let vault_y_account = AccountSharedData::new(0, 0, &Pubkey::default());
    let mint_lp_account = AccountSharedData::new(0, 0, &Pubkey::default());
    let config_account = AccountSharedData::new(0, 0, &Pubkey::default());

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        bytes_of::<Initialize>(&Initialize {
            seed,
            fee: 100,
            authority: crate::ID,
            padding: [0; 6],
        }),
        vec![
            AccountMeta::new(initializer, true),
            AccountMeta::new_readonly(mint_x, false),
            AccountMeta::new_readonly(mint_y, false),
            AccountMeta::new_readonly(mint_lp, false),
            AccountMeta::new_readonly(vault_x, false),
            AccountMeta::new_readonly(vault_y, false),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (
                initializer,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (mint_x, mint_x_account),
            (mint_y, mint_y_account),
            (mint_lp, mint_lp_account),
            (vault_x, vault_x_account),
            (vault_y, vault_y_account),
            (config, config_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );
    assert!(matches!(result.program_result, ProgramResult::Success))
}

#[test]
fn deposit() {
    // Add our built program binary
    let mut mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_amm_2024");

    // Set our seed
    let seed: u64 = 1337;

    // Programs
    mollusk.add_program(&spl_token::ID, "src/tests/spl_token-3.5.0");
    let (token_program, token_program_account) = (spl_token::ID, program_account(&spl_token::ID));
    let (system_program, system_program_account) = program::system_program();

    // Accounts
    let initializer = Pubkey::new_from_array([0x01; 32]);
    let mint_x = Pubkey::new_from_array([0x02; 32]);
    let mint_y = Pubkey::new_from_array([0x03; 32]);
    let config = Pubkey::find_program_address(&[b"config", &seed.to_le_bytes()], &crate::ID).0;
    let mint_lp = Pubkey::find_program_address(&[config.as_ref()], &crate::ID).0;
    let vault_x = Pubkey::find_program_address(&[mint_x.as_ref(), config.as_ref()], &crate::ID).0;
    let vault_y = Pubkey::find_program_address(&[mint_y.as_ref(), config.as_ref()], &crate::ID).0;

    // Fill out our account data
    let mut mint_x_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x05; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_x_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut mint_y_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x06; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_y_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut mint_lp_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x07; 32])),
            supply: 0,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_lp_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut vault_x_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_x,
            owner: config,
            amount: 0,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_x_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut vault_y_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_y,
            owner: config,
            amount: 0,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_y_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut config_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(crate::state::Config::LEN),
        crate::state::Config::LEN,
        &crate::ID,
    );
    solana_program::program_pack::Pack::pack(
        Config {

        },
        config_account.data_as_mut_slice(),
    )
