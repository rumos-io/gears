use std::{
    collections::{BTreeMap, HashSet},
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use database::Database;
use trees::iavl::Tree;

use crate::{error::StoreError, TREE_CACHE_SIZE};

#[derive(Debug, Clone, Default)]
pub struct StoreCache {
    block: BTreeMap<Vec<u8>, Vec<u8>>,
    tx: BTreeMap<Vec<u8>, Vec<u8>>,

    delete: HashSet<Vec<u8>>,
}

impl StoreCache {
    fn tx_upgrade_to_block(&mut self) {
        let tx_map = std::mem::take(&mut self.tx);

        for (key, value) in tx_map {
            let _ = self.block.insert(key, value);
        }
    }

    fn commit(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        let tx_map = std::mem::take(&mut self.tx);
        let mut block_map = std::mem::take(&mut self.block);

        block_map.extend(tx_map);
        (block_map, std::mem::take(&mut self.delete))
    }
}

#[derive(Debug)]
pub struct KVStore<KSK, DB> {
    pub(crate) persistent_store: Tree<DB>,
    store_cache: StoreCache,
    _kind_marker: PhantomData<KSK>,
}

impl<KSK, DB: Database> KVStore<KSK, DB> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, StoreError> {
        Ok(KVStore {
            persistent_store: Tree::new(
                db,
                target_version,
                TREE_CACHE_SIZE.try_into().expect("tree cache size is > 0"),
            )?,
            store_cache: Default::default(),
            _kind_marker: PhantomData,
        })
    }
}

impl<DB: Database> KVStore<CommitKVStore, DB> {
    pub fn commit(&mut self) {
        let (cache, to_delete) = self.store_cache.commit();

        for (key, value) in cache {
            if to_delete.contains(&key) {
                continue;
            }

            self.persistent_store.set(key, value);
        }

        for key in to_delete {
            let _ = self.persistent_store.remove(&key);
        }
    }
}



pub trait StoreKind {}

#[derive(Debug)]
pub struct CommitKVStore;

#[derive(Debug)]
pub struct CachedKVStore;

impl StoreKind for CommitKVStore {}

impl StoreKind for CachedKVStore {}
