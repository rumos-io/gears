use std::ops::RangeBounds;

use database::Database;

use crate::{
    range::Range,
    types::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    QueryableKVStore, TransactionalKVStore,
};

use super::{commit::CommitKVStore, KVStore};

/// Mutable variant of `KVStore`
#[derive(Debug)]
pub struct KVStoreMut<'a, DB>(pub(crate) &'a mut CommitKVStore<DB>);

impl<'a, DB: Database> KVStoreMut<'a, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.0.delete(k)
    }

    pub fn to_immutable(&self) -> KVStore<'_, DB> {
        KVStore(super::KVStoreBackend::Commit(self.0))
    }
}

impl<'a, DB: Database> QueryableKVStore<'a, DB> for KVStoreMut<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.0.get(k)
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> ImmutablePrefixStore<'a, DB> {
        self.0.prefix_store(prefix)
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        self.0.range(range)
    }
}

impl<'a, DB: Database> TransactionalKVStore<'a, DB> for KVStoreMut<'a, DB> {
    fn prefix_store_mut(self, prefix: impl IntoIterator<Item = u8>) -> MutablePrefixStore<'a, DB> {
        MutablePrefixStore {
            store: self,
            prefix: prefix.into_iter().collect(),
        }
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        self.0.set(key, value)
    }
}

impl<'a, DB> From<&'a mut CommitKVStore<DB>> for KVStoreMut<'a, DB> {
    fn from(value: &'a mut CommitKVStore<DB>) -> Self {
        Self(value)
    }
}
