use database::Database;

use crate::{range::Range, QueryableKVStore};

use self::commit::CommitKVStore;

use super::{prefix::immutable::ImmutablePrefixStore, query::kv::QueryKVStore};

pub mod cache;
pub mod commit;
pub mod mutable;

#[derive(Debug)]
pub(crate) enum KVStoreBackend<'a, DB> {
    Commit(&'a CommitKVStore<DB>),
    Query(&'a QueryKVStore<'a, DB>),
}

#[derive(Debug)]
pub struct KVStore<'a, DB>(pub(crate) KVStoreBackend<'a, DB>);

impl<'a, DB: Database> QueryableKVStore<'a, DB> for KVStore<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.get(k),
            KVStoreBackend::Query(var) => var.get(k),
        }
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStore<'a, DB> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.prefix_store(prefix),
            KVStoreBackend::Query(var) => var.prefix_store(prefix),
        }
    }

    fn range<R: std::ops::RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.range(range),
            KVStoreBackend::Query(var) => var.range(range),
        }
    }
}

impl<'a, DB> From<&'a CommitKVStore<DB>> for KVStore<'a, DB> {
    fn from(value: &'a CommitKVStore<DB>) -> Self {
        Self(KVStoreBackend::Commit(value))
    }
}

impl<'a, DB> From<&'a QueryKVStore<'a, DB>> for KVStore<'a, DB> {
    fn from(value: &'a QueryKVStore<'a, DB>) -> Self {
        Self(KVStoreBackend::Query(value))
    }
}
