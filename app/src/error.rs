use std::num::TryFromIntError;

#[derive(Debug)]
pub enum IAVLError {
    RotateError,
}

#[derive(Debug)]
pub enum AppError {
    InvalidAddress,
}

impl From<TryFromIntError> for AppError {
    fn from(_: TryFromIntError) -> AppError {
        AppError::InvalidAddress
    }
}
