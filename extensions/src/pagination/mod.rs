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

    fn paginate(
        self,
        pagination: impl Into<Pagination>,
    ) -> (PaginationResult, impl Iterator<Item = Self::Item>);

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> (Option<PaginationResult>, impl Iterator<Item = Self::Item>);
}

impl<T: Iterator<Item = U>, U: PaginationKey + Clone> IteratorPaginate for T {
    type Item = U;

    fn paginate(
        self,
        pagination: impl Into<Pagination>,
    ) -> (PaginationResult, impl Iterator<Item = Self::Item>) {
        let Pagination(variant) = pagination.into();
        match variant {
            PaginationVariant::Offset(pagination) => {
                let (PaginationByOffsetResult { total, next_key }, iter) =
                    self.paginate_by_offset(pagination);
                (
                    PaginationResult {
                        total,
                        next_key: next_key.map(|this| this.iterator_key().into_owned()),
                    },
                    TwoIterators::First(iter),
                )
            }
            PaginationVariant::Key(pagination) => {
                let (result, iter) = self.paginate_by_key(pagination);
                (result, TwoIterators::Second(iter))
            }
        }
    }

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> (Option<PaginationResult>, impl Iterator<Item = Self::Item>) {
        match pagination {
            Some(pagination) => {
                let (result, iter) = self.paginate(pagination);
                (Some(result), TwoIterators::First(iter))
            }
            None => (None, TwoIterators::Second(self)),
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

pub trait UnwrapPagination<I> {
    /// Drop pagination info and paginated return iterator only
    fn unwrap_pagination(self) -> I;
}

impl<T, I: Iterator<Item = T>> UnwrapPagination<I> for (Option<PaginationResultElement<T>>, I) {
    fn unwrap_pagination(self) -> I {
        let (_, iter) = self;
        iter
    }
}

impl<T, I: Iterator<Item = T>> UnwrapPagination<I> for (PaginationResultElement<T>, I) {
    fn unwrap_pagination(self) -> I {
        let (_, iter) = self;
        iter
    }
}

pub type PaginationResult = PaginationResultElement<Vec<u8>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationResultElement<T> {
    pub total: usize,
    pub next_key: Option<T>,
}

impl<T> PaginationResultElement<T> {
    pub fn new(total: usize, next_element: Option<T>) -> Self {
        Self {
            total,
            next_key: next_element,
        }
    }
}
