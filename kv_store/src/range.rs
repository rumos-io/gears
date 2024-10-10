use std::{borrow::Cow, ops::RangeBounds};

use database::Database;
use trees::iavl;

use crate::utils::MergedRange;

#[derive(Debug, Clone)]
pub enum Range<'a, DB, RB, R> {
    Merged(MergedRange<'a>),
    Tree(iavl::Range<'a, DB, RB, R>),
    MergedRev(std::iter::Rev<MergedRange<'a>>),
    TreeRev(std::iter::Rev<iavl::Range<'a, DB, RB, R>>),
}

impl<'a, DB: Database, R: RangeBounds<RB>, RB: AsRef<[u8]>> Range<'a, DB, RB, R> {
    pub fn rev_iter(self) -> Range<'a, DB, RB, R> {
        match self {
            Range::Merged(range) => Range::MergedRev(range.rev()),
            Range::Tree(range) => Range::TreeRev(range.rev()),
            Range::MergedRev(rev) => Range::MergedRev(rev),
            Range::TreeRev(rev_range) => Range::TreeRev(rev_range),
        }
    }
}

impl<'a, DB: Database, R: RangeBounds<RB>, RB: AsRef<[u8]>> Iterator for Range<'a, DB, RB, R> {
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

impl<'a, DB, R: RangeBounds<RB>, RB: AsRef<[u8]>> From<MergedRange<'a>> for Range<'a, DB, RB, R> {
    fn from(value: MergedRange<'a>) -> Self {
        Self::Merged(value)
    }
}

impl<'a, DB, R: RangeBounds<RB>, RB: AsRef<[u8]>> From<iavl::Range<'a, DB, RB, R>>
    for Range<'a, DB, RB, R>
{
    fn from(value: iavl::Range<'a, DB, RB, R>) -> Self {
        Self::Tree(value)
    }
}

impl<'a, DB, R: RangeBounds<RB>, RB: AsRef<[u8]>> From<std::iter::Rev<MergedRange<'a>>>
    for Range<'a, DB, RB, R>
{
    fn from(value: std::iter::Rev<MergedRange<'a>>) -> Self {
        Self::MergedRev(value)
    }
}

impl<'a, DB, R: RangeBounds<RB>, RB: AsRef<[u8]>> From<std::iter::Rev<iavl::Range<'a, DB, RB, R>>>
    for Range<'a, DB, RB, R>
{
    fn from(value: std::iter::Rev<iavl::Range<'a, DB, RB, R>>) -> Self {
        Self::TreeRev(value)
    }
}
