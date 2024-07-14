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
        let key = key.into_iter().collect::<Vec<_>>();

        if key.is_empty() {
            panic!("tried to set empty key to store")
        }

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

    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, DB> {
        let cached_values = self
            .cache
            .storage
            .range(range.clone())
            .map(|(first, second)| (Cow::Borrowed(first), Cow::Borrowed(second)));

        let tree = self.persistent.read().expect(POISONED_LOCK);
        let persisted_values = tree
            .range(range)
            // NOTE: Keys filtered only for persisted 'cause cache structure should remove inserted values on delete, but if this change then it's a place for a bug
            .filter(|(key, _)| !self.cache.delete.contains(&**key))
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
    use std::collections::BTreeMap;

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

    #[test]
    fn range_work_for_persist_values() {
        let mut tree = build_tree();

        let values_insert = [
            (1, 11),
            (2, 22),
            (3, 33),
            (4, 44),
            (5, 55),
            (6, 66),
            (7, 77),
            (8, 88),
            (9, 99),
            (10, 100),
        ]
        .into_iter()
        .map(|(key, value)| (vec![key], vec![value]))
        .collect::<BTreeMap<_, _>>();

        for (key, value) in values_insert.clone() {
            tree.set(key, value);
        }

        let range = vec![4]..vec![8];

        let expected_range = values_insert
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .range(range.clone())
            .map(|(key, value)| (Cow::Owned(key.clone()), Cow::Owned(value.clone())))
            .collect::<BTreeMap<_, _>>();

        let store = build_store(tree, None);

        // ---
        let range = store.range(range).collect::<BTreeMap<_, _>>();

        // ---
        assert_eq!(expected_range, range);
    }

    #[test]
    fn range_work_for_persist_and_cached_values() {
        let mut tree = build_tree();

        for (key, value) in [
            (1, 11),
            (2, 22),
            (3, 33),
            (4, 44),
            (5, 55),
            (6, 66),
            (8, 88),
            (9, 99),
            (10, 100),
        ] {
            tree.set(vec![key], vec![value]);
        }

        let mut cache = KVCache::default();

        cache.storage.insert(vec![6], vec![60]); // Overrides old value
        cache.storage.insert(vec![7], vec![77]); // Adds new value

        let range = vec![4]..vec![8];

        let store = build_store(tree, Some(cache));

        // ---
        let result_range = store.range(range.clone()).collect::<BTreeMap<_, _>>();

        // ---

        let expected_range = [
            (vec![4_u8], vec![44_u8]),
            (vec![5], vec![55]),
            (vec![6], vec![60]),
            (vec![7], vec![77]),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>()
        .range(range)
        .map(|(key, value)| (Cow::Owned(key.clone()), Cow::Owned(value.clone())))
        .collect::<BTreeMap<_, _>>();

        assert_eq!(expected_range, result_range);
    }

    #[test]
    fn range_work_for_persist_values_without_deleted() {
        let mut tree = build_tree();

        for (key, value) in [
            (1, 11),
            (2, 22),
            (3, 33),
            (4, 44),
            (5, 55),
            (6, 66),
            (7, 77),
            (8, 88),
            (9, 99),
            (10, 100),
        ] {
            tree.set(vec![key], vec![value]);
        }

        let mut cache = KVCache::default();

        cache.delete.insert(vec![5]);
        cache.delete.insert(vec![6]);

        let range = vec![4]..vec![8];

        let store = build_store(tree, Some(cache));

        // ---
        let result_range = store.range(range.clone()).collect::<BTreeMap<_, _>>();

        // ---

        let expected_range = [(vec![4_u8], vec![44_u8]), (vec![7], vec![77])]
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .range(range)
            .map(|(key, value)| (Cow::Owned(key.clone()), Cow::Owned(value.clone())))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(expected_range, result_range);
    }

    #[test]
    fn range_work_for_persist_and_cached_values_without_deleted() {
        let mut tree = build_tree();

        for (key, value) in [
            (1, 11),
            (2, 22),
            (3, 33),
            (4, 44),
            (5, 55),
            (6, 66),
            (7, 77),
            (8, 88),
            (9, 99),
            (10, 100),
        ] {
            tree.set(vec![key], vec![value]);
        }

        let mut cache = KVCache::default();

        cache.storage.insert(vec![4], vec![40]);
        cache.delete.insert(vec![5]);
        cache.delete.insert(vec![6]);

        let range = vec![4]..vec![8];

        let store = build_store(tree, Some(cache));

        // ---
        let result_range = store.range(range.clone()).collect::<BTreeMap<_, _>>();

        // ---

        let expected_range = [(vec![4_u8], vec![40_u8]), (vec![7], vec![77])]
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .range(range)
            .map(|(key, value)| (Cow::Owned(key.clone()), Cow::Owned(value.clone())))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(expected_range, result_range);
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
