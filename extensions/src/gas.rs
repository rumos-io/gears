use std::fmt::Debug;

pub const NO_GAS_IN_CTX: &str = "Context shouldn't have any gas so it's safe to unwrap";

pub trait GasResultExt {
    type Output;

    fn unwrap_gas(self) -> Self::Output;
}

pub trait UnwrapGasError: Debug {}

impl<T, U: UnwrapGasError> GasResultExt for Result<T, U> {
    type Output = T;

    fn unwrap_gas(self) -> Self::Output {
        self.expect(NO_GAS_IN_CTX)
    }
}
