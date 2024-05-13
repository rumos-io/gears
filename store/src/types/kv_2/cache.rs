use std::collections::{BTreeMap, HashSet};

use database::Database;

use crate::types::prefix_v2::immutable::ImmutablePrefixStoreV2;

use super::{immutable::KVStoreV2, KVStorage};

pub struct CacheKind;

impl<DB: Database> KVStorage<DB, CacheKind> {
    pub fn commit(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        self.cache.take()
    }

    pub fn clear(&mut self) {
        self.cache.storage.clear();
        self.cache.delete.clear();
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        &self,
        prefix: I,
    ) -> ImmutablePrefixStoreV2<'_, DB> {
        ImmutablePrefixStoreV2 {
            store: KVStoreV2::from(self),
            prefix: prefix.into_iter().collect(),
        }
    }
}
