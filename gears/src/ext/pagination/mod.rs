mod key;
mod offset;

pub use self::key::*;
pub use self::offset::*;

#[derive(Debug, Clone)]
pub(crate) enum PaginationVariant {
    Offset(PaginationByOffset),
    Key(PaginationByKey),
}

#[derive(Debug, Clone)]
pub struct Pagination(pub(crate) PaginationVariant);

impl From<PaginationByOffset> for Pagination {
    fn from(value: PaginationByOffset) -> Self {
        Self(PaginationVariant::Offset(value))
    }
}

impl From<PaginationByKey> for Pagination {
    fn from(value: PaginationByKey) -> Self {
        Self(PaginationVariant::Key(value))
    }
}

pub trait IteratorPaginate {
    type Item;

    fn paginate(self, pagination: impl Into<Pagination>) -> impl Iterator<Item = Self::Item>;

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item>;

    fn skip_by_pagination(
        self,
        pagination: impl Into<Pagination>,
    ) -> impl Iterator<Item = Self::Item>;

    fn maybe_skip_by_pagination<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item>;
}

impl<T: Iterator<Item = U>, U: PaginationKeyIterator> IteratorPaginate for T {
    type Item = U;

    fn paginate(self, pagination: impl Into<Pagination>) -> impl Iterator<Item = Self::Item> {
        let Pagination(variant) = pagination.into();
        match variant {
            PaginationVariant::Offset(pagination) => {
                TwoIterators::First(self.paginate_by_offset(pagination))
            }
            PaginationVariant::Key(pagination) => {
                TwoIterators::Second(self.paginate_by_key(pagination))
            }
        }
    }

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => TwoIterators::First(self.paginate(pagination)),
            None => TwoIterators::Second(self),
        }
    }

    fn skip_by_pagination(
        self,
        pagination: impl Into<Pagination>,
    ) -> impl Iterator<Item = Self::Item> {
        let Pagination(variant) = pagination.into();
        match variant {
            PaginationVariant::Offset(pagination) => {
                TwoIterators::First(self.skip_by_offset_pagination(pagination))
            }
            PaginationVariant::Key(pagination) => {
                TwoIterators::Second(self.skip_by_key_pagination(pagination))
            }
        }
    }

    fn maybe_skip_by_pagination<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => TwoIterators::First(self.skip_by_pagination(pagination)),
            None => TwoIterators::Second(self),
        }
    }
}

#[derive(Debug, Clone)]
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
