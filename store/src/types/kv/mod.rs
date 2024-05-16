use std::{
    borrow::Cow,
    marker::PhantomData,
    ops::RangeBounds,
    sync::{Arc, RwLock},
};

use database::Database;
use trees::iavl::Tree;

use crate::{error::POISONED_LOCK, range::Range, utils::MergedRange};

use self::store_cache::KVCache;

pub mod cache;
pub mod commit;
pub mod immutable;
pub mod mutable;
pub mod store_cache;

#[derive(Debug)]
pub struct KVBank<DB, SK> {
    pub(crate) persistent: Arc<RwLock<Tree<DB>>>,
    pub(crate) cache: KVCache,
    _marker: PhantomData<SK>,
}

impl<DB: Database, SK> KVBank<DB, SK> {
    #[inline]
    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.persistent.read().expect(POISONED_LOCK).root_hash()
    }

    #[inline]
    pub fn last_committed_version(&self) -> u32 {
        self.persistent
            .read()
            .expect(POISONED_LOCK)
            .loaded_version()
    }

    #[inline]
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.cache
            .delete(k)
            .or(self.persistent.read().expect(POISONED_LOCK).get(k))
    }

    #[inline]
    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        self.cache.set(key, value)
    }

    pub fn clear_cache(&mut self) {
        self.cache.storage.clear();
        self.cache.delete.clear();
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.cache.get(k.as_ref()) {
            Ok(var) => var,
            Err(_) => return None,
        }
        .cloned()
        .or(self.persistent.read().expect(POISONED_LOCK).get(k.as_ref()))
    }

    // TODO:NOW You could iterate over values that should have been deleted
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        let cached_values = self
            .cache
            .storage
            .range(range.clone())
            .map(|(first, second)| (Cow::Borrowed(first), Cow::Borrowed(second)));

        let tree = self.persistent.read().expect(POISONED_LOCK);
        let persisted_values = tree
            .range(range)
            .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second)));

        MergedRange::merge(cached_values, persisted_values).into()
    }

    pub fn caches_update(&mut self, KVCache { storage, delete }: KVCache) {
        self.cache.storage.extend(storage);
        self.cache.delete.extend(delete);
    }
}

#[cfg(test)]
mod tests {
    use database::MemDB;

    use crate::TREE_CACHE_SIZE;

    use super::*;

    #[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TestStore;

    #[test]
    fn delete_empty_cache() {
        let mut tree = build_tree();

        let key = vec![1];

        tree.set(key.clone(), vec![2]);

        let mut store = build_store(tree, None);

        // ---
        let deleted = store.delete(&key);

        // ---
        assert_eq!(Some(vec![2]), deleted);
    }

    #[test]
    fn delete_taken_from_cache() {
        let mut tree = build_tree();

        let key = vec![1];

        tree.set(key.clone(), vec![2]);

        let mut cache = KVCache::default();

        cache.storage.insert(key.clone(), vec![3]);

        let mut store = build_store(tree, Some(cache));

        // ---
        let deleted = store.delete(&key);

        // ---
        assert_eq!(Some(vec![3]), deleted);
    }

    #[test]
    fn get_empty_cache() {
        let mut tree = build_tree();

        let key = vec![1];

        tree.set(key.clone(), vec![2]);

        let store = build_store(tree, None);

        // ---
        let result = store.get(&key);

        // ---
        assert_eq!(Some(vec![2]), result);
    }

    #[test]
    fn get_from_cache() {
        let mut tree = build_tree();

        let key = vec![1];

        tree.set(key.clone(), vec![2]);

        let mut cache = KVCache::default();

        cache.storage.insert(key.clone(), vec![3]);

        let store = build_store(tree, Some(cache));

        // ---
        let result = store.get(&key);

        // ---
        assert_eq!(Some(vec![3]), result);
    }

    #[test]
    fn get_deleted() {
        let mut tree = build_tree();

        let key = vec![1];

        tree.set(key.clone(), vec![2]);

        let mut cache = KVCache::default();

        cache.delete.insert(key.clone());

        let store = build_store(tree, Some(cache));

        // ---
        let result = store.get(&key);

        // ---
        assert_eq!(None, result);
    }

    fn build_tree() -> Tree<MemDB> {
        Tree::new(
            MemDB::new(),
            None,
            TREE_CACHE_SIZE
                .try_into()
                .expect("Unreachable. Tree cache size is > 0"),
        )
        .expect("Failed to create Tree")
    }

    fn build_store(tree: Tree<MemDB>, cache: Option<KVCache>) -> KVBank<MemDB, TestStore> {
        KVBank {
            persistent: Arc::new(RwLock::new(tree)),
            cache: cache.unwrap_or_default(),
            _marker: PhantomData,
        }
    }
}
