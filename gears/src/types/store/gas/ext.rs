use super::errors::GasStoreErrors;

pub const NO_GAS_IN_CTX: &str = "Context shouldn't have any gas so it's safe to unwrap";

pub trait GasResultExt {
    type Output;

    fn unwrap_gas(self) -> Self::Output;
}

impl<T> GasResultExt for Result<T, GasStoreErrors> {
    type Output = T;

    fn unwrap_gas(self) -> Self::Output {
        self.expect(NO_GAS_IN_CTX)
    }
}
