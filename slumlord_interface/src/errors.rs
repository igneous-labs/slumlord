use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum SlumlordError {
    #[error("No succeeding CheckRepaid instruction found")]
    NoSucceedingCheckRepaid = 0,
    #[error("Can only Borrow once before CheckRepaid")]
    BorrowAlreadyActive = 1,
    #[error("Outstanding loan was not fully repaid")]
    InsufficientRepay = 2,
}
impl From<SlumlordError> for ProgramError {
    fn from(e: SlumlordError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for SlumlordError {
    fn type_of() -> &'static str {
        "SlumlordError"
    }
}
impl PrintProgramError for SlumlordError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
