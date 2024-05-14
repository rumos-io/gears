use std::ops::RangeBounds;

use database::Database;
use trees::iavl::QueryTree;

use crate::types::prefix::immutable::ImmutablePrefixStore;

pub struct QueryKVStore<'a, DB>(QueryTree<'a, DB>);

impl<'a, DB: Database> QueryKVStore<'a, DB> {
    pub fn new(tree: QueryTree<'a, DB>) -> Self {
        Self(tree)
    }
}

impl<DB: Database> QueryKVStore<'_, DB> {
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(
        &self,
        range: R,
    ) -> crate::range::Range<'_, R, DB> {
        self.0.range(range).into()
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.0.get(k.as_ref())
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        &self,
        prefix: I,
    ) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: self.into(),
            prefix: prefix.into_iter().collect(),
        }
    }
}
