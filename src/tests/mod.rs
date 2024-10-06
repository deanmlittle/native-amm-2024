use crate::{Config, Deposit, Initialize, Withdraw};
use bytemuck::bytes_of;
use core::mem;
use mollusk_svm::{
    program::{self, program_account},
    result::ProgramResult,
    Mollusk,
};
use solana_program::{
    clock::Clock, instruction::AccountMeta, instruction::Instruction, program_option::COption,
    program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar,
};
use solana_sdk::account::{AccountSharedData, WritableAccount};

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
        &[&[0u8], bytes_of::<Initialize>(&Initialize {
            seed,
            fee: 100,
            authority: crate::ID,
            padding: [0; 6],
        })].concat(),
        vec![
            AccountMeta::new(initializer, true),
            AccountMeta::new_readonly(mint_x, false),
            AccountMeta::new_readonly(mint_y, false),
            AccountMeta::new(mint_lp, false),
            AccountMeta::new(vault_x, false),
            AccountMeta::new(vault_y, false),
            AccountMeta::new(config, false),
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
#[ignore]
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
    let user = Pubkey::new_from_array([0x01; 32]);
    let mint_x = Pubkey::new_from_array([0x02; 32]);
    let mint_y = Pubkey::new_from_array([0x03; 32]);
    let user_x = Pubkey::new_from_array([0x04; 32]);
    let user_y = Pubkey::new_from_array([0x05; 32]);
    let user_lp = Pubkey::new_from_array([0x06; 32]);
    let (config, config_bump) =
        Pubkey::find_program_address(&[b"config", &seed.to_le_bytes()], &crate::ID);
    let (mint_lp, lp_bump) = Pubkey::find_program_address(&[config.as_ref()], &crate::ID);
    let (vault_x, x_bump) =
        Pubkey::find_program_address(&[mint_x.as_ref(), config.as_ref()], &crate::ID);
    let (vault_y, y_bump) =
        Pubkey::find_program_address(&[mint_y.as_ref(), config.as_ref()], &crate::ID);

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

    let mut user_x_account = AccountSharedData::new(
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
            owner: user_x,
            amount: 1_000_000,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        user_x_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut user_y_account = AccountSharedData::new(
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
            owner: user_y,
            amount: 1_000_000,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        user_y_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut user_lp_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_lp,
            owner: user_lp,
            amount: 0,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        user_lp_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut config_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(mem::size_of::<Config>()),
        mem::size_of::<Config>(),
        &crate::ID,
    );
    config_account.set_data_from_slice(bytes_of::<Config>(&Config {
        seed,
        authority: crate::ID,
        mint_x,
        mint_y,
        fee: 100u16,
        locked: 0,
        config_bump,
        lp_bump,
        x_bump,
        y_bump,
        padding: [0],
    }));

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        bytes_of::<Deposit>(&Deposit {
            amount: 1_000_000,
            max_x: 1_000_000,
            max_y: 1_000_000,
            expiration: Clock::get()
                .unwrap()
                .unix_timestamp
                .checked_add(100_000)
                .unwrap(),
        }),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(mint_x, false),
            AccountMeta::new_readonly(mint_y, false),
            AccountMeta::new_readonly(mint_lp, false),
            AccountMeta::new(user_x, false),
            AccountMeta::new(user_y, false),
            AccountMeta::new(user_lp, false),
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
                user,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (mint_x, mint_x_account),
            (mint_y, mint_y_account),
            (mint_lp, mint_lp_account),
            (user_x, user_x_account),
            (user_y, user_y_account),
            (user_lp, user_lp_account),
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
#[ignore = "reason"]
fn withdraw() {
    // Add our built program binary
    let mut mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_amm_2024");

    // Set our seed
    let seed: u64 = 1337;

    // Programs
    mollusk.add_program(&spl_token::ID, "src/tests/spl_token-3.5.0");
    let (token_program, token_program_account) = (spl_token::ID, program_account(&spl_token::ID));
    let (system_program, system_program_account) = program::system_program();

    // Accounts
    let user = Pubkey::new_from_array([0x01; 32]);
    let mint_x = Pubkey::new_from_array([0x02; 32]);
    let mint_y = Pubkey::new_from_array([0x03; 32]);
    let user_x = Pubkey::new_from_array([0x04; 32]);
    let user_y = Pubkey::new_from_array([0x05; 32]);
    let user_lp = Pubkey::new_from_array([0x06; 32]);
    let (config, config_bump) =
        Pubkey::find_program_address(&[b"config", &seed.to_le_bytes()], &crate::ID);
    let (mint_lp, lp_bump) = Pubkey::find_program_address(&[config.as_ref()], &crate::ID);
    let (vault_x, x_bump) =
        Pubkey::find_program_address(&[mint_x.as_ref(), config.as_ref()], &crate::ID);
    let (vault_y, y_bump) =
        Pubkey::find_program_address(&[mint_y.as_ref(), config.as_ref()], &crate::ID);

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
            supply: 1_000_000,
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
            amount: 1_000_000,
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
            amount: 1_000_000,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_y_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut user_x_account = AccountSharedData::new(
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
            owner: user_x,
            amount: 0,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        user_x_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut user_y_account = AccountSharedData::new(
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
            owner: user_y,
            amount: 0,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        user_y_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut user_lp_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_lp,
            owner: user_lp,
            amount: 1_000_000,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        user_lp_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut config_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(mem::size_of::<Config>()),
        mem::size_of::<Config>(),
        &crate::ID,
    );
    config_account.set_data_from_slice(bytes_of::<Config>(&Config {
        seed,
        authority: crate::ID,
        mint_x,
        mint_y,
        fee: 100u16,
        locked: 0,
        config_bump,
        lp_bump,
        x_bump,
        y_bump,
        padding: [0],
    }));

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        bytes_of::<Withdraw>(&Withdraw {
            amount: 1_000_000,
            min_x: 500_000,
            min_y: 500_000,
            expiration: Clock::get()
                .unwrap()
                .unix_timestamp
                .checked_add(100_000)
                .unwrap(),
        }),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(mint_x, false),
            AccountMeta::new_readonly(mint_y, false),
            AccountMeta::new_readonly(mint_lp, false),
            AccountMeta::new(user_x, false),
            AccountMeta::new(user_y, false),
            AccountMeta::new(user_lp, false),
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
                user,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (mint_x, mint_x_account),
            (mint_y, mint_y_account),
            (mint_lp, mint_lp_account),
            (user_x, user_x_account),
            (user_y, user_y_account),
            (user_lp, user_lp_account),
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
#[ignore = "reason"]
fn lock() {
    // Add our built program binary
    let mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_amm_2024");

    // Set our seed
    let seed: u64 = 1337;

    let authority = Pubkey::new_from_array([0x01; 32]);
    let (config, config_bump) =
        Pubkey::find_program_address(&[b"config", &seed.to_le_bytes()], &crate::ID);

    let mut config_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(mem::size_of::<Config>()),
        mem::size_of::<Config>(),
        &crate::ID,
    );
    config_account.set_data_from_slice(bytes_of::<Config>(&Config {
        seed,
        authority,
        mint_x: Pubkey::new_from_array([0x02; 32]),
        mint_y: Pubkey::new_from_array([0x03; 32]),
        fee: 100u16,
        locked: 0,
        config_bump,
        lp_bump: 0,
        x_bump: 0,
        y_bump: 0,
        padding: [0],
    }));

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        &0u8.to_le_bytes(),
        vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(config, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (
                authority,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (config, config_account),
        ],
    );
    assert!(matches!(result.program_result, ProgramResult::Success))
}