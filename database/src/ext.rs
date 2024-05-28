use std::fmt::Debug;

pub const DATABASE_CORRUPTION_MSG: &str = "invalid data in database - possible database corruption";

pub trait UnwrapCorrupt {
    type Output;

    fn unwrap_or_corrupt(self) -> Self::Output;
}

impl<T> UnwrapCorrupt for Option<T> {
    type Output = T;

    fn unwrap_or_corrupt(self) -> Self::Output {
        self.expect(DATABASE_CORRUPTION_MSG)
    }
}

impl<T, E: Debug> UnwrapCorrupt for Result<T, E> {
    type Output = T;

    fn unwrap_or_corrupt(self) -> Self::Output {
        self.expect(DATABASE_CORRUPTION_MSG)
    }
}
