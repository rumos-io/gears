use std::borrow::Cow;

use database::Database;
use trees::iavl::Range as TreeRange;

use crate::utils::MergedRange;

#[derive(Debug, Clone)]
pub enum Range<'a, DB> {
    Merged(MergedRange<'a>),
    Tree(TreeRange<'a, DB>),
}

impl<'a, DB: Database> Iterator for Range<'a, DB> {
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

impl<'a, DB> From<MergedRange<'a>> for Range<'a, DB> {
    fn from(value: MergedRange<'a>) -> Self {
        Self::Merged(value)
    }
}

impl<'a, DB> From<TreeRange<'a, DB>> for Range<'a, DB> {
    fn from(value: TreeRange<'a, DB>) -> Self {
        Self::Tree(value)
    }
}
