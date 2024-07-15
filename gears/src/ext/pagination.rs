#[derive(Debug, Clone)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
}

impl From<(usize, usize)> for Pagination {
    fn from((offset, limit): (usize, usize)) -> Self {
        Self { offset, limit }
    }
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

pub enum TwoIterators<I, T: Iterator<Item = I>, U: Iterator<Item = I>> {
    One(T),
    Second(U),
}

impl<I, T: Iterator<Item = I>, U: Iterator<Item = I>> Iterator for TwoIterators<I, T, U> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TwoIterators::One(var) => var.next(),
            TwoIterators::Second(var) => var.next(),
        }
    }
}
