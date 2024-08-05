use database::Database;

use crate::store::kv::mutable::KVStoreMut;

use super::immutable::ImmutablePrefixStore;

/// Wraps an mutable KVStore with a prefix
#[derive(Debug)]
pub struct MutablePrefixStore<'a, DB> {
    pub(crate) store: KVStoreMut<'a, DB>,
    pub(crate) prefix: Vec<u8>,
}

impl<DB> MutablePrefixStore<'_, DB> {
    pub fn to_immutable(&self) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: self.store.to_immutable(),
            prefix: self.prefix.clone(),
        }
    }
}

impl<DB: Database> MutablePrefixStore<'_, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k].concat();
        self.store.delete(&full_key)
    }

    pub fn get<T: AsRef<[u8]> + ?Sized>(&self, k: &T) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k.as_ref()].concat();
        self.store.get(&full_key)
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(&mut self, k: KI, v: VI) {
        // TODO: do we need to check for zero length keys as with the KVStore::set?
        let full_key = [self.prefix.clone(), k.into_iter().collect()].concat();
        self.store.set(full_key, v);
    }
}
