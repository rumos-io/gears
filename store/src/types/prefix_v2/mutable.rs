use database::Database;

use crate::{
    types::kv_2::mutable::KVStoreMutV2, QueryableKVStoreV2, ReadPrefixStore,
    TransactionalKVStoreV2, WritePrefixStore,
};

pub struct MutablePrefixStoreV2<'a, DB> {
    pub(crate) store: KVStoreMutV2<'a, DB>,
    pub(crate) prefix: Vec<u8>,
}

impl<'a, DB: Database> MutablePrefixStoreV2<'a, DB> {
    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k].concat();
        self.store.get(&full_key)
    }

    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.store.delete(k)
    }
}

impl<DB: Database> ReadPrefixStore for MutablePrefixStoreV2<'_, DB> {
    fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k.as_ref()].concat();
        self.store.get(&full_key)
    }
}

impl<DB: Database> WritePrefixStore for MutablePrefixStoreV2<'_, DB> {
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, k: KI, v: VI) {
        // TODO: do we need to check for zero length keys as with the KVStore::set?
        let full_key = [self.prefix.clone(), k.into_iter().collect()].concat();
        self.store.set(full_key, v)
    }
}
