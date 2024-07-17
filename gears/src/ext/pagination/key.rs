use std::borrow::Cow;

use vec1::Vec1;

use crate::types::{base::coin::UnsignedCoin, store::gas::errors::GasStoreErrors};

use super::TwoIterators;

#[derive(Debug, Clone)]
pub struct PaginationByKey {
    pub key: Vec1<u8>,
    pub limit: usize,
}

impl From<(Vec1<u8>, usize)> for PaginationByKey {
    fn from((key, limit): (Vec1<u8>, usize)) -> Self {
        Self { key, limit }
    }
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

    fn skip_by_key_pagination(
        self,
        pagination: impl Into<PaginationByKey>,
    ) -> impl Iterator<Item = Self::Item>;

    fn maybe_skip_by_key_pagination<P: Into<PaginationByKey>>(
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

    fn skip_by_key_pagination(
        self,
        pagination: impl Into<PaginationByKey>,
    ) -> impl Iterator<Item = Self::Item> {
        let PaginationByKey { key, limit: _ } = pagination.into();

        self.skip_while(move |this| this.iterator_key().as_ref() != key)
    }

    fn maybe_skip_by_key_pagination<P: Into<PaginationByKey>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => TwoIterators::First(self.skip_by_key_pagination(pagination)),
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

impl PaginationKeyIterator for Vec<u8> {
    fn iterator_key(&self) -> impl AsRef<[u8]> {
        AsRef::<[u8]>::as_ref(self)
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

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use vec1::vec1;

    const VALUE_VALID: &'static str = "default value for test if valid";

    #[test]
    fn all_values() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let expected: Vec1<_> = vec1![vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let result: Vec1<_> = array
            .into_iter()
            .paginate_by_key((vec1![1], 6))
            .collect::<Vec<_>>()
            .try_into()
            .expect(VALUE_VALID);

        assert_eq!(expected, result)
    }

    #[test]
    fn first_half_of_values() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let expected = vec1![vec![1_u8], vec![2], vec![3]];

        let result: Vec1<_> = array
            .into_iter()
            .paginate_by_key((vec1![1], 3))
            .collect::<Vec<_>>()
            .try_into()
            .expect(VALUE_VALID);

        assert_eq!(expected, result)
    }

    #[test]
    fn second_half_of_values() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let expected = vec1![vec![4], vec![5], vec![6]];

        let result: Vec1<_> = array
            .into_iter()
            .paginate_by_key((vec1![4], 3))
            .collect::<Vec<_>>()
            .try_into()
            .expect(VALUE_VALID);

        assert_eq!(expected, result)
    }

    #[test]
    fn first_middle_of_values() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let expected = vec1![vec![2], vec![3]];

        let result: Vec1<_> = array
            .into_iter()
            .paginate_by_key((vec1![2], 2))
            .collect::<Vec<_>>()
            .try_into()
            .expect(VALUE_VALID);

        assert_eq!(expected, result)
    }

    #[test]
    fn second_middle_of_values() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let expected = vec1![vec![4], vec![5]];

        let result: Vec1<_> = array
            .into_iter()
            .paginate_by_key((vec1![4], 2))
            .collect::<Vec<_>>()
            .try_into()
            .expect(VALUE_VALID);

        assert_eq!(expected, result)
    }
}
