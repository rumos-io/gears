use std::fmt::Debug;

pub const TESTING_MSG: &str = "unwrap value in test";

pub trait UnwrapTesting {
    type Output;

    fn unwrap_test(self) -> Self::Output;
}

impl<T> UnwrapTesting for Option<T> {
    type Output = T;

    fn unwrap_test(self) -> Self::Output {
        self.expect(TESTING_MSG)
    }
}

impl<T, E: Debug> UnwrapTesting for Result<T, E> {
    type Output = T;

    fn unwrap_test(self) -> Self::Output {
        self.expect(TESTING_MSG)
    }
}
