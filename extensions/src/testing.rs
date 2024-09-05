use std::fmt::Debug;

pub const TESTING_MSG: &str = "unwrap value in test";

pub trait UnwrapCorrupt {
    type Output;

    fn testing(self) -> Self::Output;
}

impl<T> UnwrapCorrupt for Option<T> {
    type Output = T;

    fn testing(self) -> Self::Output {
        self.expect(TESTING_MSG)
    }
}

impl<T, E: Debug> UnwrapCorrupt for Result<T, E> {
    type Output = T;

    fn testing(self) -> Self::Output {
        self.expect(TESTING_MSG)
    }
}
