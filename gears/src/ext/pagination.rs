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

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item>;
}

impl<T: Iterator<Item = U>, U> IteratorPaginate for T {
    type Item = U;

    fn paginate(self, pagination: impl Into<Pagination>) -> impl Iterator<Item = Self::Item> {
        let Pagination { offset, limit } = pagination.into();
        self.skip(offset * limit).take(limit)
    }

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => {
                let Pagination { offset, limit } = pagination.into();
                TwoIterators::First(self.skip(offset * limit).take(limit))
            }
            None => TwoIterators::Second(self),
        }
    }
}

enum TwoIterators<I, T: Iterator<Item = I>, U: Iterator<Item = I>> {
    First(T),
    Second(U),
}

impl<I, T: Iterator<Item = I>, U: Iterator<Item = I>> Iterator for TwoIterators<I, T, U> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TwoIterators::First(var) => var.next(),
            TwoIterators::Second(var) => var.next(),
        }
    }
}
