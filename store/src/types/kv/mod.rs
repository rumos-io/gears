use database::Database;

use crate::{range::Range, QueryableKVStore};

use self::commit::CommitKVStore;

use super::prefix::immutable::ImmutablePrefixStore;

pub mod cache;
pub mod commit;
pub mod mutable;

pub struct KVStore<'a, DB>(pub(crate) &'a CommitKVStore<DB>);

impl<DB: Database> QueryableKVStore<DB> for KVStore<'_, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.0.get(k)
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(&self, prefix: I) -> ImmutablePrefixStore<'_, DB> {
        self.0.prefix_store(prefix)
    }

    fn range<R: std::ops::RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        self.0.range(range)
    }
}

impl<'a, DB> From<&'a CommitKVStore<DB>> for KVStore<'a, DB> {
    fn from(value: &'a CommitKVStore<DB>) -> Self {
        Self(value)
    }
}
