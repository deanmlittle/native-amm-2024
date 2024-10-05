use bytemuck::{Pod, Zeroable};
use native_amm_macros::TryFromBytes;
use solana_program::program_error::ProgramError;
pub enum EscrowInstructions {
    Initialize,
    Deposit,
    Withdraw,
    Swap,
    Freeze,
    Lock
}

impl TryFrom<&u8> for EscrowInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Deposit),
            2 => Ok(Self::Withdraw),
            3 => Ok(Self::Swap),
            4 => Ok(Self::Freeze),
            5 => Ok(Self::Lock),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Deposit {
    amount: u64, // Amount of LP token to claim
    max_x: u64, // Max amount of X we are willing to deposit
    max_y: u64, // Max amount of Y we are willing to deposit
    expiration: i64,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Swap {
    amount: u64, // Amount of tokens we deposit
    min: u64, // Minimum amount of tokens I'd be willing to withdraw
    expiration: i64
}