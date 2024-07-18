use std::borrow::Cow;

use itertools::Itertools;
use vec1::Vec1;

use crate::types::{base::coin::UnsignedCoin, store::gas::errors::GasStoreErrors};

use super::{PaginationResultElement, TwoIterators};

pub type PaginationByKeyResult = PaginationResultElement<Vec<u8>>;

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
    fn iterator_key(&self) -> Cow<'_, [u8]>;
}

pub trait IteratorPaginateByKey {
    type Item;

    fn paginate_by_key(
        self,
        pagination: impl Into<PaginationByKey>,
    ) -> (PaginationByKeyResult, impl Iterator<Item = Self::Item>);

    fn maybe_paginate_by_key<P: Into<PaginationByKey>>(
        self,
        pagination: Option<P>,
    ) -> (
        Option<PaginationByKeyResult>,
        impl Iterator<Item = Self::Item>,
    );
}

impl<T: Iterator<Item = U>, U: PaginationKeyIterator> IteratorPaginateByKey for T {
    type Item = U;

    fn paginate_by_key(
        self,
        pagination: impl Into<PaginationByKey>,
    ) -> (PaginationByKeyResult, impl Iterator<Item = Self::Item>) {
        let PaginationByKey { key, limit } = pagination.into();

        let mut iterator =
            itertools::peek_nth(self.skip_while(move |this| this.iterator_key().as_ref() != key));

        let last = iterator
            .peek_nth(limit)
            .map(|e| e.iterator_key().into_owned());
        let count = match iterator.try_len() {
            Ok(len) => len,
            Err((_lower_bound, upper_bound)) => upper_bound.unwrap_or(usize::MAX),
        };

        (
            PaginationResultElement::new(count, last),
            iterator.take(limit),
        )
    }

    fn maybe_paginate_by_key<P: Into<PaginationByKey>>(
        self,
        pagination: Option<P>,
    ) -> (
        Option<PaginationByKeyResult>,
        impl Iterator<Item = Self::Item>,
    ) {
        match pagination {
            Some(pagination) => {
                let (result, iter) = self.paginate_by_key(pagination);
                (Some(result), TwoIterators::First(iter))
            }
            None => (None, TwoIterators::Second(self)),
        }
    }
}

impl PaginationKeyIterator for UnsignedCoin {
    fn iterator_key(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.denom.as_ref())
    }
}

impl<T> PaginationKeyIterator for (Cow<'_, Vec<u8>>, T) {
    fn iterator_key(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_ref())
    }
}

impl PaginationKeyIterator for Cow<'_, Vec<u8>> {
    fn iterator_key(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.as_ref())
    }
}

impl PaginationKeyIterator for Vec<u8> {
    fn iterator_key(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<T: PaginationKeyIterator> PaginationKeyIterator for Result<T, GasStoreErrors> {
    fn iterator_key(&self) -> Cow<'_, [u8]> {
        match self {
            Ok(var) => var.iterator_key(),
            Err(var) => Cow::Borrowed(&var.key),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::UnwrapPagination;

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
            .unwrap_pagination()
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
            .unwrap_pagination()
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
            .unwrap_pagination()
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
            .unwrap_pagination()
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
            .unwrap_pagination()
            .collect::<Vec<_>>()
            .try_into()
            .expect(VALUE_VALID);

        assert_eq!(expected, result)
    }

    #[test]
    fn p_result_all_values() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let (p_result, _) = array.into_iter().paginate_by_key((vec1![1], 2));

        let expected = PaginationResultElement::new(6, Some(vec![3]));

        assert_eq!(expected, p_result);
    }

    #[test]
    fn p_result_last_value() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let (p_result, _) = array.into_iter().paginate_by_key((vec1![6], 2));

        let expected = PaginationResultElement::new(1, None);

        assert_eq!(expected, p_result);
    }

    #[test]
    fn p_result_not_existed_value() {
        let array = [vec![1_u8], vec![2], vec![3], vec![4], vec![5], vec![6]];

        let (p_result, _) = array.into_iter().paginate_by_key((vec1![7], 2));

        let expected = PaginationResultElement::new(0, None);

        assert_eq!(expected, p_result);
    }
}
