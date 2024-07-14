/// Copy from: [try_map](https://docs.rs/try_map/latest/src/try_map/lib.rs.html#13-19) crate \
/// This method still not a part of std(but have [plans](https://github.com/rust-lang/rfcs/issues/1815) for it) so I copied it temporally
///
/// Extend `Option` with a fallible map method
///
/// This is useful for mapping fallible operations (i.e. operations that)
/// return `Result`, over an optional type. The result will be
/// `Result<Option<U>>`, which makes it easy to handle the errors that
/// originate from inside the closure that's being mapped.
///
/// # Type parameters
///
/// - `T`: The input `Option`'s value type
/// - `U`: The outputs `Option`'s value type
/// - `E`: The possible error during the mapping
pub trait FallibleMapExt<T, U, E> {
    /// Try to apply a fallible map function to the option
    fn try_map<F>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>;
}

impl<T, U, E> FallibleMapExt<T, U, E> for Option<T> {
    fn try_map<F>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match self {
            Some(x) => f(x).map(Some),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
}

pub trait IteratorPaginate {
    type Item;

    fn paginate(self, pagination: impl Into<Pagination>) -> impl Iterator<Item = Self::Item>;
}

impl<T: Iterator<Item = U>, U> IteratorPaginate for T {
    type Item = U;

    fn paginate(self, pagination: impl Into<Pagination>) -> impl Iterator<Item = Self::Item> {
        let Pagination { offset, limit } = pagination.into();
        self.skip(offset * limit).take(limit)
    }
}
