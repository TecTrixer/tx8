use std::{array::TryFromSliceError, error::Error, fmt::Display, str::Utf8Error};
#[derive(Clone, Copy, Debug)]
pub enum Tx8Error {
    ParseError,
    InstructionError,
    OutOfBoundsWrite,
    InvalidRegister,
    InvalidSysCall,
    InvalidOpCode(u8),
    DivisionByZero,
    NoInputGiven,
}

impl Error for Tx8Error {}

impl Display for Tx8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tx8Error::InvalidOpCode(op) => write!(f, "InvalidOpCode({:x})", op),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl From<TryFromSliceError> for Tx8Error {
    fn from(_value: TryFromSliceError) -> Self {
        Tx8Error::ParseError
    }
}
impl From<Utf8Error> for Tx8Error {
    fn from(_value: Utf8Error) -> Self {
        Tx8Error::ParseError
    }
}
