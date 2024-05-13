use std::sync::Arc;

use database::Database;

use crate::{error::POISONED_LOCK, types::prefix_v2::immutable::ImmutablePrefixStoreV2};

use super::{cache::CacheKind, immutable::KVStoreV2, KVStorage};

pub struct CommitKind;

impl<DB: Database> KVStorage<DB, CommitKind> {
    pub fn commit(&mut self) -> [u8; 32] {
        let (cache, delete) = self.cache.take();
        let mut persistent = self.persistent.write().expect(POISONED_LOCK);

        for (key, value) in cache {
            if delete.contains(&key) {
                continue;
            }

            persistent.set(key, value);
        }

        for key in delete {
            let _ = persistent.remove(&key);
        }

        let (hash, _) = persistent.save_version().ok().unwrap_or_default(); //TODO: is it safe to assume this won't ever error?
        hash
    }

    pub fn to_cache_kind(&self) -> KVStorage<DB, CacheKind> {
        KVStorage {
            persistent: Arc::clone(&self.persistent),
            cache: Default::default(), // TODO:NOW Should cache be ignored?
            _marker: std::marker::PhantomData,
        }
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
