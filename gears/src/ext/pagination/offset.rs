use super::TwoIterators;

#[derive(Debug, Clone)]
pub struct PaginationByOffset {
    pub offset: usize,
    pub limit: usize,
}

impl From<(usize, usize)> for PaginationByOffset {
    fn from((offset, limit): (usize, usize)) -> Self {
        Self { offset, limit }
    }
}

pub trait IteratorPaginateByOffset {
    type Item;

    fn paginate_by_offset(
        self,
        pagination: impl Into<PaginationByOffset>,
    ) -> impl Iterator<Item = Self::Item>;

    fn maybe_paginate_by_offset<P: Into<PaginationByOffset>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item>;
}

impl<T: Iterator<Item = U>, U> IteratorPaginateByOffset for T {
    type Item = U;

    fn paginate_by_offset(
        self,
        pagination: impl Into<PaginationByOffset>,
    ) -> impl Iterator<Item = Self::Item> {
        let PaginationByOffset { offset, limit } = pagination.into();
        self.skip(offset * limit).take(limit)
    }

    fn maybe_paginate_by_offset<P: Into<PaginationByOffset>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => TwoIterators::First(self.paginate_by_offset(pagination)),
            None => TwoIterators::Second(self),
        }
    }
}
