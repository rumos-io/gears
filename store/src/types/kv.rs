use std::{collections::BTreeMap, ops::RangeBounds};

use database::Database;
use trees::iavl::{Range, Tree};

use crate::{error::Error, ReadKVStore, WriteKVStore, TREE_CACHE_SIZE};

use super::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore};

#[derive(Debug)]
pub struct KVStore<DB> {
    pub(crate) persistent_store: Tree<DB>,
    block_cache: BTreeMap<Vec<u8>, Vec<u8>>,
    tx_cache: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl<DB: Database> WriteKVStore<DB> for KVStore<DB> {
    fn prefix_store_mut(
        &mut self,
        prefix: impl IntoIterator<Item = u8>,
    ) -> MutablePrefixStore<'_, DB> {
        MutablePrefixStore {
            store: self,
            prefix: prefix.into_iter().collect(),
        }
    }

    /// Returns default if failed to save cache
    fn commit(&mut self) -> [u8; 32] {
        self.write_then_clear_tx_cache();
        self.write_then_clear_block_cache();
        let (hash, _) = self
            .persistent_store
            .save_version()
            .ok()
            .unwrap_or_default(); //TODO: is it safe to assume this won't ever error?
        hash
    }

    fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        let key: Vec<u8> = key.into_iter().collect();

        if key.is_empty() {
            // TODO: copied from SDK, need to understand why this is needed and maybe create a type which captures the restriction
            panic!("key is empty")
        }

        self.tx_cache.insert(key, value.into_iter().collect());
    }
}

impl<DB: Database> ReadKVStore<DB> for KVStore<DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        let tx_cache_val = self.tx_cache.get(k.as_ref());

        if tx_cache_val.is_none() {
            let block_cache_val = self.block_cache.get(k.as_ref());

            if block_cache_val.is_none() {
                return self.persistent_store.get(k.as_ref());
            };

            return block_cache_val.cloned();
        }

        tx_cache_val.cloned()
    }

    fn prefix_store(&self, prefix: Vec<u8>) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: self.into(),
            prefix: prefix.into_iter().collect(),
        }
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        //TODO: this doesn't iterate over cached values
        // let tx_cached_values = self.tx_cache.range(range.clone());
        // let block_cached_values = self.block_cache.range(range.clone());
        // let persisted_values = self.persistent_store.range(range.clone());

        // MergedRange::merge(
        //     tx_cached_values,
        //     MergedRange::merge(block_cached_values, persisted_values),
        // );

        self.persistent_store.range(range)
    }

    // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>> {
    //     self.persistent_store
    //         .range(..)
    //         .map(|(key, _value)| key)
    //         .filter(|key| key.starts_with(key_prefix.as_ref()))
    //         .collect()
    // }
}

impl<DB: Database> KVStore<DB> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, Error> {
        Ok(KVStore {
            persistent_store: Tree::new(
                db,
                target_version,
                TREE_CACHE_SIZE.try_into().expect("tree cache size is > 0"),
            )?,
            block_cache: BTreeMap::new(),
            tx_cache: BTreeMap::new(),
        })
    }

    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        let tx_value = self.tx_cache.remove(k);
        let block_value = self.block_cache.remove(k);
        let persisted_value = self.persistent_store.remove(k);

        tx_value.or(block_value).or(persisted_value)
    }

    /// Writes tx cache into block cache then clears the tx cache
    pub fn write_then_clear_tx_cache(&mut self) {
        let mut keys: Vec<&Vec<u8>> = self.tx_cache.keys().collect();
        keys.sort();

        for key in keys {
            let value = self
                .tx_cache
                .get(key)
                .expect("key is definitely in the HashMap");
            self.block_cache.insert(key.to_owned(), value.to_owned());
        }
        self.tx_cache.clear();
    }

    /// Clears the tx cache
    pub fn clear_tx_cache(&mut self) {
        self.tx_cache.clear();
    }

    /// Writes block cache into the tree store then clears the block cache
    fn write_then_clear_block_cache(&mut self) {
        let mut keys: Vec<&Vec<u8>> = self.block_cache.keys().collect();
        keys.sort();

        for key in keys {
            let value = self
                .block_cache
                .get(key)
                .expect("key is definitely in the HashMap");
            self.persistent_store.set(key.to_owned(), value.to_owned())
        }
        self.block_cache.clear();
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.persistent_store.root_hash()
    }

    pub fn last_committed_version(&self) -> u32 {
        self.persistent_store.loaded_version()
    }
}
