use std::ops::RangeBounds;

use database::Database;

use crate::{
    range::Range,
    types::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    QueryableKVStore, TransactionalKVStore,
};

use super::commit::CommitKVStore;

pub struct KVStoreMut<'a, DB>(pub(crate) &'a mut CommitKVStore<DB>);

impl<DB: Database> KVStoreMut<'_, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.0.delete(k)
    }

    // /// Commit changes from tx cache to block on successful tx
    // fn commit(&mut self) {
    //     self.0.cache.tx_upgrade_to_block()
    // }
}

impl<DB: Database> QueryableKVStore<DB> for KVStoreMut<'_, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.0.get(k)
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(&self, prefix: I) -> ImmutablePrefixStore<'_, DB> {
        self.0.prefix_store(prefix)
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        self.0.range(range)
    }
}

impl<DB: Database> TransactionalKVStore<DB> for KVStoreMut<'_, DB> {
    fn prefix_store_mut(
        &mut self,
        prefix: impl IntoIterator<Item = u8>,
    ) -> MutablePrefixStore<'_, DB> {
        self.0.prefix_store_mut(prefix)
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        self.0.set(key, value)
    }
}
