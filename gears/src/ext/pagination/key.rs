use std::borrow::Cow;

use crate::types::{base::coin::UnsignedCoin, store::gas::errors::GasStoreErrors};

use super::TwoIterators;

#[derive(Debug, Clone)]
pub struct PaginationByKey {
    pub key: Vec<u8>,
    pub limit: usize,
}

pub trait PaginationKeyIterator {
    fn iterator_key(&self) -> impl AsRef<[u8]>;
}

pub trait IteratorPaginateByKey {
    type Item;

    fn paginate_by_key(
        self,
        pagination: impl Into<PaginationByKey>,
    ) -> impl Iterator<Item = Self::Item>;

    fn maybe_paginate_by_key<P: Into<PaginationByKey>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item>;
}

impl<T: Iterator<Item = U>, U: PaginationKeyIterator> IteratorPaginateByKey for T {
    type Item = U;

    fn paginate_by_key(
        self,
        pagination: impl Into<PaginationByKey>,
    ) -> impl Iterator<Item = Self::Item> {
        let PaginationByKey { key, limit } = pagination.into();
        self.skip_while(move |this| this.iterator_key().as_ref() != key)
            .take(limit)
    }

    fn maybe_paginate_by_key<P: Into<PaginationByKey>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => TwoIterators::First(self.paginate_by_key(pagination)),
            None => TwoIterators::Second(self),
        }
    }
}

impl PaginationKeyIterator for UnsignedCoin {
    fn iterator_key(&self) -> impl AsRef<[u8]> {
        AsRef::<[u8]>::as_ref(&self.denom)
    }
}

impl<T> PaginationKeyIterator for (Cow<'_, Vec<u8>>, T) {
    fn iterator_key(&self) -> impl AsRef<[u8]> {
        self.0.as_ref()
    }
}

impl PaginationKeyIterator for Cow<'_, Vec<u8>> {
    fn iterator_key(&self) -> impl AsRef<[u8]> {
        self.as_ref()
    }
}

impl<T: PaginationKeyIterator> PaginationKeyIterator for Result<T, GasStoreErrors> {
    fn iterator_key(&self) -> impl AsRef<[u8]> {
        match self {
            Ok(var) => TwoAsRef::First(var.iterator_key()),
            Err(var) => TwoAsRef::Second(&var.key),
        }
    }
}

enum TwoAsRef<T: AsRef<[u8]>, U: AsRef<[u8]>> {
    First(T),
    Second(U),
}

impl<T: AsRef<[u8]>, U: AsRef<[u8]>> AsRef<[u8]> for TwoAsRef<T, U> {
    fn as_ref(&self) -> &[u8] {
        match self {
            TwoAsRef::First(var) => var.as_ref(),
            TwoAsRef::Second(var) => var.as_ref(),
        }
    }
}
