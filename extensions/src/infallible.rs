use std::convert::Infallible;

pub trait UnwrapInfallible {
    type Output;

    fn unwrap_infallible(self) -> Self::Output;
}

impl<T> UnwrapInfallible for Result<T, Infallible> {
    type Output = T;

    fn unwrap_infallible(self) -> Self::Output {
        self.expect("Infallible")
    }
}
