use std::{
    collections::{BTreeMap, HashSet},
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use database::Database;
use trees::iavl::Tree;

use crate::{error::StoreError, TREE_CACHE_SIZE};

pub trait StoreKind {}

#[derive(Debug)]
pub struct Commit;

impl StoreKind for Commit {}

impl StoreKind for Cache {}

#[derive(Debug, Clone, Default)]
pub struct Cache {
    block: BTreeMap<Vec<u8>, Vec<u8>>,
    tx: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl Cache {
    fn tx_upgrade_to_block(&mut self) {
        let tx_map = std::mem::take(&mut self.tx);

        for (key, value) in tx_map {
            let _ = self.block.insert(key, value);
        }
    }

    fn commit(&mut self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let tx_map = std::mem::take(&mut self.tx);
        let mut block_map = std::mem::take(&mut self.block);

        block_map.extend(tx_map);
        block_map
    }
}

#[derive(Debug)]
pub struct KVStore<KSK, DB> {
    pub(crate) persistent_store: Tree<DB>,
    delete_cache: HashSet<Vec<u8>>,
    ext: KSK,
}

impl<KSK, DB: Database> KVStore<KSK, DB> {
    pub fn new(db: DB, ext: KSK, target_version: Option<u32>) -> Result<Self, StoreError> {
        Ok(KVStore {
            persistent_store: Tree::new(
                db,
                target_version,
                TREE_CACHE_SIZE.try_into().expect("tree cache size is > 0"),
            )?,
            delete_cache: HashSet::new(),
            ext,
        })
    }
}

impl<DB: Database> KVStore<Commit, DB> {
    pub fn commit(&mut self, cache: &mut Cache) {
        let cache = cache.commit();
        let delete = std::mem::take(&mut self.delete_cache);

        for (key, value) in cache {
            if delete.contains(&key) {
                continue;
            }

            self.persistent_store.set(key, value);
        }

        for key in delete {
            let _ = self.persistent_store.remove(&key);
        }
    }

    pub fn delete(&mut self, k: impl IntoIterator<Item = u8>) -> Option<Vec<u8>> {
        let key = k.into_iter().collect::<Vec<_>>();

        let persisted_value = self.persistent_store.get(&key);
        let _ = self.delete_cache.insert(key);

        persisted_value
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        if self.delete_cache.contains(key) {
            None
        } else {
            self.persistent_store.get(key)
        }
    }
}

impl<DB: Database> KVStore<Cache, DB> {
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        let tx_value = self.ext.tx.remove(k);
        let block_value = self.ext.block.remove(k);

        let persisted_value = if tx_value.is_none() || block_value.is_none() {
            let persisted_value = self.persistent_store.get(k);
            if persisted_value.is_some() {
                let _ = self.delete_cache.insert(k.to_owned());
            }

            persisted_value
        } else {
            None
        };

        tx_value.or(block_value).or(persisted_value)
    }

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        if self.delete_cache.contains(k.as_ref()) {
            return None;
        }

        self.ext
            .tx
            .get(k.as_ref())
            .or(self.ext.block.get(k.as_ref()))
            .cloned()
            .or(self.persistent_store.get(k.as_ref()))
    }
}
