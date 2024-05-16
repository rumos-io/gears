use std::collections::{BTreeMap, HashSet};

use database::Database;

use crate::{types::prefix::immutable::ImmutablePrefixStore, CacheKind};

use super::{immutable::KVStore, KVBank};

impl<DB: Database> KVBank<DB, CacheKind> {
    pub fn commit(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        self.cache.take()
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        &self,
        prefix: I,
    ) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: KVStore::from(self),
            prefix: prefix.into_iter().collect(),
        }
    }
}
