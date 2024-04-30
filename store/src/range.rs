use std::{borrow::Cow, ops::RangeBounds};

use database::Database;
use trees::iavl::Range as TreeRange;

use crate::utils::MergedRange;

pub enum Range<'a, R: RangeBounds<Vec<u8>>, DB> {
    Merged(MergedRange<'a>),
    Tree(TreeRange<'a, R, DB>),
}

impl<'a, R: RangeBounds<Vec<u8>>, DB: Database> Iterator for Range<'a, R, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Range::Merged(range) => range.next(),
            Range::Tree(range) => range
                .next()
                .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second))),
        }
    }
}

impl<'a, R: RangeBounds<Vec<u8>>, DB> From<MergedRange<'a>> for Range<'a, R, DB> {
    fn from(value: MergedRange<'a>) -> Self {
        Self::Merged(value)
    }
}

impl<'a, R: RangeBounds<Vec<u8>>, DB> From<TreeRange<'a, R, DB>> for Range<'a, R, DB> {
    fn from(value: TreeRange<'a, R, DB>) -> Self {
        Self::Tree(value)
    }
}
