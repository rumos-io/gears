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

    fn skip_by_offset_pagination(
        self,
        pagination: impl Into<PaginationByOffset>,
    ) -> impl Iterator<Item = Self::Item>;

    fn maybe_skip_by_offset_pagination<P: Into<PaginationByOffset>>(
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

    fn skip_by_offset_pagination(
        self,
        pagination: impl Into<PaginationByOffset>,
    ) -> impl Iterator<Item = Self::Item> {
        let PaginationByOffset { offset, limit } = pagination.into();
        self.skip(offset * limit)
    }

    fn maybe_skip_by_offset_pagination<P: Into<PaginationByOffset>>(
        self,
        pagination: Option<P>,
    ) -> impl Iterator<Item = Self::Item> {
        match pagination {
            Some(pagination) => TwoIterators::First(self.skip_by_offset_pagination(pagination)),
            None => TwoIterators::Second(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_values() {
        let array = (0..20).collect::<Vec<_>>();

        let expected = (0..20).collect::<Vec<_>>();

        let result = array
            .into_iter()
            .paginate_by_offset((0, 20))
            .collect::<Vec<_>>();

        assert_eq!(expected, result)
    }

    #[test]
    fn first_half_of_values() {
        let array = (0..20).collect::<Vec<_>>();

        let expected = (0..10).collect::<Vec<_>>();

        let result = array
            .into_iter()
            .paginate_by_offset((0, 10))
            .collect::<Vec<_>>();

        assert_eq!(expected, result)
    }

    #[test]
    fn second_half_of_values() {
        let array = (0..20).collect::<Vec<_>>();

        let expected = (10..20).collect::<Vec<_>>();

        let result = array
            .into_iter()
            .paginate_by_offset((1, 10))
            .collect::<Vec<_>>();

        assert_eq!(expected, result)
    }

    #[test]
    fn first_middle_of_values() {
        let array = (0..20).collect::<Vec<_>>();

        let expected = (5..10).collect::<Vec<_>>();

        let result = array
            .into_iter()
            .paginate_by_offset((1, 5))
            .collect::<Vec<_>>();

        assert_eq!(expected, result)
    }

    #[test]
    fn second_middle_of_values() {
        let array = (0..20).collect::<Vec<_>>();

        let expected = (10..15).collect::<Vec<_>>();

        let result = array
            .into_iter()
            .paginate_by_offset((2, 5))
            .collect::<Vec<_>>();

        assert_eq!(expected, result)
    }
}
