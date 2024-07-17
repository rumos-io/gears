mod key;
mod offset;

use crate::types::pagination::response::PaginationResponse;

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
    ) -> (
        PaginationResult<Self::Item>,
        impl Iterator<Item = Self::Item>,
    );

    fn maybe_paginate<P: Into<Pagination>>(
        self,
        pagination: Option<P>,
    ) -> (
        Option<PaginationResult<Self::Item>>,
        impl Iterator<Item = Self::Item>,
    );
}

impl<T: Iterator<Item = U>, U: PaginationKeyIterator + Clone> IteratorPaginate for T {
    type Item = U;

    fn paginate(
        self,
        pagination: impl Into<Pagination>,
    ) -> (
        PaginationResult<Self::Item>,
        impl Iterator<Item = Self::Item>,
    ) {
        let Pagination(variant) = pagination.into();
        match variant {
            PaginationVariant::Offset(pagination) => {
                let (result, iter) = self.paginate_by_offset(pagination);
                (result, TwoIterators::First(iter))
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
    ) -> (
        Option<PaginationResult<Self::Item>>,
        impl Iterator<Item = Self::Item>,
    ) {
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

impl<T, I: Iterator<Item = T>> UnwrapPagination<I> for (Option<PaginationResult<T>>, I) {
    fn unwrap_pagination(self) -> I {
        let (_, iter) = self;
        iter
    }
}

impl<T, I: Iterator<Item = T>> UnwrapPagination<I> for (PaginationResult<T>, I) {
    fn unwrap_pagination(self) -> I {
        let (_, iter) = self;
        iter
    }
}

#[derive(Debug, Clone)]
pub struct PaginationResult<T> {
    pub total: usize,
    pub next_element: Option<T>,
}

impl<T> PaginationResult<T> {
    pub fn new(total: usize, next_element: Option<T>) -> Self {
        Self {
            total,
            next_element,
        }
    }
}

pub fn bytes_pagination_result<T: PaginationKeyIterator>(
    p_result: Option<PaginationResult<T>>,
) -> Option<PaginationResult<Vec<u8>>> {
    match p_result {
        Some(PaginationResult {
            total,
            next_element,
        }) => Some(PaginationResult::new(
            total,
            next_element.map(|this| this.iterator_key().into_owned()),
        )),
        None => None,
    }
}

impl<T: PaginationKeyIterator> From<PaginationResult<T>> for PaginationResponse {
    fn from(
        PaginationResult {
            total,
            next_element,
        }: PaginationResult<T>,
    ) -> Self {
        Self {
            next_key: next_element
                .map(|this| this.iterator_key().into_owned())
                .unwrap_or_default(),
            total: total as u64,
        }
    }
}
