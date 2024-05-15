use std::error::Error;

pub const INVALID_TENDERMINT_DATA: &str = "invalid data received from Tendermint";

pub trait UnwrapInvalid {
    type Output;

    fn unwrap_or_invalid(self) -> Self::Output;
}

impl<T, E: Error> UnwrapInvalid for Result<T, E> {
    type Output = T;

    fn unwrap_or_invalid(self) -> Self::Output {
        match self {
            Ok(value) => value,
            Err(e) => panic!("{} - {}", INVALID_TENDERMINT_DATA, e),
        }
    }
}
