use bytemuck::{Pod, Zeroable};
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
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct Make {
    pub seed: u64,
    pub amount: u64,
    pub receive: u64,
}

impl TryFrom<&[u8]> for Make {
    
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_pod_read_unaligned::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}
