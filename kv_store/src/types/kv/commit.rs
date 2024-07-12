use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use database::Database;
use trees::iavl::Tree;

use crate::{
    error::{KVStoreError, POISONED_LOCK},
    types::prefix::immutable::ImmutablePrefixStore,
    ApplicationStore, TransactionStore, TREE_CACHE_SIZE,
};

use super::{immutable::KVStore, KVBank};

impl<DB: Database> KVBank<DB, ApplicationStore> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, KVStoreError> {
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

        let (hash, _) = persistent.save_version().ok().unwrap_or_default(); //TODO:ME is it safe to assume this won't ever error?
        hash
    }

    pub fn to_cache_kind(&self) -> KVBank<DB, TransactionStore> {
        KVBank {
            persistent: Arc::clone(&self.persistent),
            cache: self.cache.clone(),
            _marker: std::marker::PhantomData,
        }
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
