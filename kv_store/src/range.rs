use std::{borrow::Cow, ops::Bound};

use database::Database;
use trees::iavl;

use crate::utils::MergedRange;

#[derive(Debug, Clone)]
pub enum Range<'a, DB> {
    Merged(MergedRange<'a>),
    Tree(iavl::Range<'a, DB>),
    MergedRev(std::iter::Rev<MergedRange<'a>>),
    TreeRev(iavl::RevRange<'a, DB, Vec<u8>, (Bound<Vec<u8>>, Bound<Vec<u8>>)>),
}

impl<'a, DB> Range<'a, DB> {
    pub fn rev_iter(self) -> Range<'a, DB> {
        match self {
            Range::Merged(range) => Range::MergedRev(range.rev()),
            Range::Tree(range) => Range::TreeRev(range.rev_iter()),
            Range::MergedRev(rev) => Range::MergedRev(rev),
            Range::TreeRev(rev_range) => Range::TreeRev(rev_range),
        }
    }
}

impl<'a, DB: Database> Iterator for Range<'a, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Range::Merged(range) => range.next(),
            Range::Tree(range) => range
                .next()
                .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second))),
            Range::MergedRev(range) => range.next(),
            Range::TreeRev(range) => range
                .next()
                .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second))),
        }
    }
}

impl<'a, DB> From<MergedRange<'a>> for Range<'a, DB> {
    fn from(value: MergedRange<'a>) -> Self {
        Self::Merged(value)
    }
}

impl<'a, DB> From<iavl::Range<'a, DB>> for Range<'a, DB> {
    fn from(value: iavl::Range<'a, DB>) -> Self {
        Self::Tree(value)
    }
}

impl<'a, DB> From<std::iter::Rev<MergedRange<'a>>> for Range<'a, DB> {
    fn from(value: std::iter::Rev<MergedRange<'a>>) -> Self {
        Self::MergedRev(value)
    }
}

impl<'a, DB> From<iavl::RevRange<'a, DB, Vec<u8>, (Bound<Vec<u8>>, Bound<Vec<u8>>)>>
    for Range<'a, DB>
{
    fn from(value: iavl::RevRange<'a, DB, Vec<u8>, (Bound<Vec<u8>>, Bound<Vec<u8>>)>) -> Self {
        Self::TreeRev(value)
    }
}
