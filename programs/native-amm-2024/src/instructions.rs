use bytemuck::{Pod, Zeroable};
use native_amm_macros::TryFromBytes;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(Clone)]
pub enum AMMInstructions {
    Initialize,
    Deposit,
    Withdraw,
    Swap,
    Lock,
}

impl TryFrom<&u8> for AMMInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Deposit),
            2 => Ok(Self::Withdraw),
            3 => Ok(Self::Swap),
            4 => Ok(Self::Lock),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

impl AMMInstructions {
    pub fn serialize<T: Pod>(&self, ix: T) -> Vec<u8> {
        [
            &[self.clone() as u8],
            bytemuck::bytes_of::<T>(&ix)
        ].concat()
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Initialize {
    pub seed: u64,
    pub fee: u16,
    pub authority: Pubkey,
    pub padding: [u8; 6],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Deposit {
    pub amount: u64, // Amount of LP token to claim
    pub max_x: u64,  // Max amount of X we are willing to deposit
    pub max_y: u64,  // Max amount of Y we are willing to deposit
    pub expiration: i64,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Withdraw {
    pub amount: u64, // Amount of LP token to burn
    pub min_x: u64,  // Min amount of X we are willing to withdraw
    pub min_y: u64,  // Min amount of Y we are willing to withdraw
    pub expiration: i64,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Swap {
    pub amount: u64, // Amount of tokens we deposit
    pub min: u64,    // Minimum amount of tokens I'd be willing to withdraw
    pub expiration: i64,
}
