use std::{borrow::Cow, collections::BTreeMap, ops::RangeBounds};

use database::Database;
use trees::iavl::Tree;

use crate::{
    error::StoreError,
    range::Range,
    types::prefix::{immutable::ImmutablePrefixStore, mutable::MutablePrefixStore},
    utils::MergedRange,
    TREE_CACHE_SIZE,
};

use super::{cache::KVStoreCache, mutable::KVStoreMut};

/// KVStore variant which has `commit` method which persist values in DB
#[derive(Debug)]
pub struct CommitKVStore<DB> {
    pub(crate) persistent_store: Tree<DB>,
    pub(crate) cache: KVStoreCache,
}

impl<DB: Database> CommitKVStore<DB> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, StoreError> {
        Ok(CommitKVStore {
            persistent_store: Tree::new(
                db,
                target_version,
                TREE_CACHE_SIZE.try_into().expect("tree cache size is > 0"),
            )?,
            cache: Default::default(),
        })
    }

    pub fn commit(&mut self) -> [u8; 32] {
        let (cache, delete) = self.cache.take();

        for (key, value) in cache {
            if delete.contains(&key) {
                continue;
            }

            self.persistent_store.set(key, value);
        }

        for key in delete {
            let _ = self.persistent_store.remove(&key);
        }

        let (hash, _) = self
            .persistent_store
            .save_version()
            .ok()
            .unwrap_or_default(); //TODO: is it safe to assume this won't ever error?
        hash
    }

    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        let _ = self.cache.delete.insert(k.to_owned());

        let tx_value = self.cache.tx.remove(k);
        let block_value = self.cache.block.remove(k);

        let persisted_value = if tx_value.is_none() || block_value.is_none() {
            self.persistent_store.get(k)
        } else {
            None
        };

        tx_value.or(block_value).or(persisted_value)
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.persistent_store.root_hash()
    }

    pub fn last_committed_version(&self) -> u32 {
        self.persistent_store.loaded_version()
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        if self.cache.delete.contains(k.as_ref()) {
            return None;
        }

        self.cache
            .tx
            .get(k.as_ref())
            .or(self.cache.block.get(k.as_ref()))
            .cloned()
            .or(self.persistent_store.get(k.as_ref()))
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        &self,
        prefix: I,
    ) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: self.into(),
            prefix: prefix.into_iter().collect(),
        }
    }

    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        let cached_values = {
            let tx_cached_values = self.cache.tx.range(range.clone());
            let mut block_cached_values = self
                .cache
                .block
                .range(range.clone())
                .collect::<BTreeMap<_, _>>();

            block_cached_values.extend(tx_cached_values);
            block_cached_values
                .into_iter()
                .map(|(first, second)| (Cow::Borrowed(first), Cow::Borrowed(second)))
        };

        let persisted_values = self
            .persistent_store
            .range(range)
            .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second)));

        MergedRange::merge(cached_values, persisted_values).into()
    }

    pub fn prefix_store_mut(
        &mut self,
        prefix: impl IntoIterator<Item = u8>,
    ) -> MutablePrefixStore<'_, DB> {
        MutablePrefixStore {
            store: KVStoreMut(self),
            prefix: prefix.into_iter().collect(),
        }
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        let key: Vec<u8> = key.into_iter().collect();

        if key.is_empty() {
            // TODO: copied from SDK, need to understand why this is needed and maybe create a type which captures the restriction
            panic!("key is empty")
        }

        self.cache.tx.insert(key, value.into_iter().collect());
    }
}
