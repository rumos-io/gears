use std::convert::Infallible;

pub trait UnwrapInfallible {
    type Output;

    fn infallible(self) -> Self::Output;
}

impl<T> UnwrapInfallible for Result<T, Infallible> {
    type Output = T;

    fn infallible(self) -> Self::Output {
        self.expect("Infallible")
    }
}
