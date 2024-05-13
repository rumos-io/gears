use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use database::Database;
use trees::iavl::Tree;

use crate::{
    error::{StoreError, POISONED_LOCK},
    types::prefix_v2::immutable::ImmutablePrefixStoreV2,
    CacheKind, CommitKind, TREE_CACHE_SIZE,
};

use super::{immutable::KVStoreV2, KVStorage};

impl<DB: Database> KVStorage<DB, CommitKind> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, StoreError> {
        Ok(Self {
            persistent: Arc::new(RwLock::new(Tree::new(
                db,
                target_version,
                TREE_CACHE_SIZE
                    .try_into()
                    .expect("Unreachable. Tree cache size is > 0"),
            )?)),
            cache: Default::default(),
            _marker: PhantomData,
        })
    }

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
