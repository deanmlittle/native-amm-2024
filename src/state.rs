use crate::utils::{
    check_eq_program_derived_address, check_eq_program_derived_address_and_get_bump,
};
use bytemuck::{Pod, Zeroable};
use native_amm_macros::TryFromBytes;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::instruction::{close_account, transfer_checked};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, TryFromBytes)]
pub struct Config {
    pub seed: u64,
    pub authority: Pubkey,
    pub mint_x: Pubkey, // Token X Mint
    pub mint_y: Pubkey, // Token Y Mint
    pub fee: u16,       // Swap fee in basis points
    pub locked: u8,
    pub config_bump: u8,
    pub lp_bump: u8,
    pub x_bump: u8,
    pub y_bump: u8,
    pub padding: [u8; 1],
}
