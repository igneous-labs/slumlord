use bytemuck::{try_from_bytes, try_from_bytes_mut};
use slumlord_interface::{
    check_repaid_ix, init_ix, BorrowKeys, CheckRepaidKeys, InitKeys, RepayKeys, Slumlord,
};
use solana_program::{
    instruction::Instruction, program_error::ProgramError, pubkey::Pubkey, system_program, sysvar,
};

pub mod program {
    sanctum_macros::declare_program_keys!(
        "s1umBj7CEUA6djs6V1c6o2Nym3QrqF4ryKDr1Nm1FKt",
        [("slumlord", b"slumlord")]
    );
}

pub const SLUMLORD_ACCOUNT_LEN: usize = std::mem::size_of::<Slumlord>();

pub const INIT_KEYS: InitKeys = InitKeys {
    slumlord: program::SLUMLORD_ID,
    system_program: system_program::ID,
};

pub const CHECK_REPAID_KEYS: CheckRepaidKeys = CheckRepaidKeys {
    slumlord: program::SLUMLORD_ID,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowFreeArgs {
    pub dst: Pubkey,
}

impl BorrowFreeArgs {
    pub fn resolve(self) -> BorrowKeys {
        BorrowKeys {
            dst: self.dst,
            slumlord: program::SLUMLORD_ID,
            instructions: sysvar::instructions::ID,
        }
    }
}

impl From<BorrowFreeArgs> for BorrowKeys {
    fn from(value: BorrowFreeArgs) -> Self {
        value.resolve()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepayFreeArgs {
    pub src: Pubkey,
}

impl RepayFreeArgs {
    pub fn resolve(self) -> RepayKeys {
        RepayKeys {
            src: self.src,
            slumlord: program::SLUMLORD_ID,
            system_program: system_program::ID,
        }
    }
}

impl From<RepayFreeArgs> for RepayKeys {
    fn from(value: RepayFreeArgs) -> Self {
        value.resolve()
    }
}

pub fn init_ix_full() -> std::io::Result<Instruction> {
    init_ix(INIT_KEYS)
}

pub fn check_repaid_ix_full() -> std::io::Result<Instruction> {
    check_repaid_ix(CHECK_REPAID_KEYS)
}

pub fn try_slumlord(slumlord_acc_data: &[u8]) -> Result<&Slumlord, ProgramError> {
    try_from_bytes(slumlord_acc_data).map_err(|_e| ProgramError::InvalidAccountData)
}

pub fn try_slumlord_mut(slumlord_acc_data: &mut [u8]) -> Result<&mut Slumlord, ProgramError> {
    try_from_bytes_mut(slumlord_acc_data).map_err(|_e| ProgramError::InvalidAccountData)
}
