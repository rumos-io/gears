use database::Database;

use crate::{types::kv::KVStore, ReadKVStore, WriteKVStore, WritePrefixStore};

/// Wraps an mutable reference to a KVStore with a prefix
pub struct MutablePrefixStore<'a, DB> {
    pub(crate) store: &'a mut KVStore<DB>,
    pub(crate) prefix: Vec<u8>,
}

impl<'a, DB: Database> MutablePrefixStore<'a, DB> {
    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k].concat();
        self.store.get(&full_key)
    }

    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.store.delete(k)
    }
}

impl<DB: Database> WritePrefixStore for MutablePrefixStore<'_, DB> {
    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, k: KI, v: VI) {
        // TODO: do we need to check for zero length keys as with the KVStore::set?
        let full_key = [self.prefix.clone(), k.into_iter().collect()].concat();
        self.store.set(full_key, v.into_iter().collect::<Vec<_>>())
    }
}
