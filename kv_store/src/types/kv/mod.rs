use std::{
    borrow::Cow,
    collections::HashMap,
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
    pub(crate) tx: KVCache,
    pub(crate) block: KVCache,
    pub(crate) _marker: PhantomData<SK>,
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
        self.tx
            .delete(k)
            .or(self.persistent.read().expect(POISONED_LOCK).get(k))
    }

    #[inline]
    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        self.tx.set(key, value)
    }

    pub fn clear_tx_cache(&mut self) {
        self.tx.storage.clear();
        self.tx.delete.clear();
    }

    pub fn clear_block_cache(&mut self) {
        self.tx.storage.clear();
        self.tx.delete.clear();
    }

    pub fn upgrade_cache(&mut self) {
        let (set_values, delete) = self.tx.take();
        for (key, value) in set_values {
            self.block.delete.remove(&key);
            self.block.set(key, value);
        }

        for del in delete {
            self.block.delete(&del);
        }
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.tx.get(k.as_ref()) {
            Ok(var) => var,
            Err(_) => return None,
        }
        .or(self.block.get(k.as_ref()).unwrap_or(None))
        .cloned()
        .or(self.persistent.read().expect(POISONED_LOCK).get(k.as_ref()))
    }

    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, DB> {
        let cached_values = self
            .block
            .storage
            .range(range.clone())
            .chain(self.tx.storage.range(range.clone()))
            .map(|(first, second)| (Cow::Borrowed(first), Cow::Borrowed(second)))
            .collect::<HashMap<_, _>>();

        let tree = self.persistent.read().expect(POISONED_LOCK);
        let persisted_values = tree
            .range(range)
            // NOTE: Keys filtered only for persisted 'cause cache structure should remove inserted values on delete, but if this change then it's a place for a bug
            .filter(|(key, _)| {
                !(self.tx.delete.contains(&**key) || self.block.delete.contains(&**key))
            })
            .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second)));

        MergedRange::merge(cached_values.into_iter(), persisted_values).into()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use database::MemDB;

    use crate::{TransactionStore, TREE_CACHE_SIZE};

    use super::*;

    const ERR_MSG: &str = "expected != actual";

    #[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TestStore;

    #[test]
    pub fn upgrade_cache_work_without_del() {
        let mut store = build_other_store::<TestStore>();
        store.set(vec![1], vec![11]);
        store.set(vec![2], vec![22]);

        store.upgrade_cache();

        let KVCache { storage, delete } = store.block;

        let expected_storage = [(vec![1], vec![11]), (vec![2], vec![22])]
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        assert_eq!(expected_storage, storage, "{ERR_MSG}");
        assert!(delete.is_empty())
    }

    #[test]
    pub fn commit_with_only_tx_for_transactional() {
        let mut store = build_other_store::<TransactionStore>();

        store.delete(&vec![2]);
        store.set(vec![1], vec![2]);

        let (insert, delete) = store.commit();

        assert!(insert.is_empty());
        assert!(delete.is_empty());
    }

    #[test]
    pub fn commit_with_tx_and_block_for_transactional() {
        let mut store = build_other_store::<TransactionStore>();

        store.delete(&[2]);
        store.set(vec![1], vec![2]);
        store.upgrade_cache();
        store.set(vec![1], vec![22]);
        store.set(vec![2], vec![33]);
        store.delete(&[3]);

        let (insert, delete) = store.commit();

        let expected_insert = [(vec![1], vec![2])].into_iter().collect::<BTreeMap<_, _>>();
        let expected_delete = [vec![2]].into_iter().collect::<HashSet<_>>();

        assert_eq!(expected_insert, insert, "{ERR_MSG}");
        assert_eq!(expected_delete, delete, "{ERR_MSG}");
    }

    #[test]
    pub fn commit_with_block_twice_upgraded_for_transactional() {
        let mut store = build_other_store::<TransactionStore>();

        store.delete(&[2]);
        store.set(vec![1], vec![2]);
        store.upgrade_cache();
        store.set(vec![1], vec![22]);
        store.set(vec![2], vec![33]);
        store.delete(&[3]);
        store.upgrade_cache();

        let (insert, delete) = store.commit();

        let expected_insert = [(vec![1], vec![22]), (vec![2], vec![33])]
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        let expected_delete = [vec![3]].into_iter().collect::<HashSet<_>>();

        assert_eq!(expected_insert, insert, "{ERR_MSG}");
        assert_eq!(expected_delete, delete, "{ERR_MSG}");
    }

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
    fn get_from_tx_cache() {
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
    fn get_from_tx_cache_deleted_in_block() {
        let mut store = build_store(build_tree(), None);

        let key = vec![1];

        store.delete(&key);
        store.upgrade_cache();

        store.set(key.clone(), vec![2]);

        // ---
        let result = store.get(&key);

        // ---
        assert_eq!(Some(vec![2]), result);
    }

    #[test]
    fn get_from_block_cache() {
        let mut tree = build_tree();

        let key = vec![1];

        tree.set(key.clone(), vec![2]);

        let mut cache = KVCache::default();

        cache.storage.insert(key.clone(), vec![3]);

        let mut store = build_store(tree, Some(cache));
        store.upgrade_cache();

        // ---
        let result = store.get(&key);

        // ---
        assert_eq!(Some(vec![3]), result);
    }

    #[test]
    fn get_from_tx_overwriting_block_cache() {
        let mut tree = build_tree();
        tree.set(vec![1], vec![2]);

        let mut cache = KVCache::default();

        cache.storage.insert(vec![1], vec![3]);

        let mut store = build_store(tree, Some(cache));
        store.upgrade_cache();
        store.set(vec![1], vec![4]);

        // ---
        let result = store.get(&vec![1]);

        // ---
        assert_eq!(Some(vec![4]), result);
    }

    #[test]
    fn get_deleted_in_tx() {
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
    fn range_work_for_persist_and_cached_values_with_block() {
        let mut tree = build_tree();

        for (key, value) in [(1, 11), (2, 22), (3, 33), (4, 44)] {
            tree.set(vec![key], vec![value]);
        }

        let mut cache = KVCache::default();

        cache.storage.insert(vec![1], vec![111]); // Overrides old value

        let range = ..vec![4];

        let mut store = build_store(tree, Some(cache));
        store.upgrade_cache();

        store.set(vec![2], vec![222]);

        // ---
        let result_range = store.range(range.clone()).collect::<BTreeMap<_, _>>();

        // ---

        let expected_range = [
            (vec![1_u8], vec![111_u8]),
            (vec![2], vec![222]),
            (vec![3], vec![33]),
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
    fn range_work_for_persist_values_without_deleted_with_block() {
        let mut tree = build_tree();

        for (key, value) in [(1, 11), (2, 22), (3, 33), (4, 44), (5, 55), (6, 66)] {
            tree.set(vec![key], vec![value]);
        }

        let mut cache = KVCache::default();

        cache.delete.insert(vec![1]);
        cache.delete.insert(vec![2]);

        let range = ..vec![6];

        let mut store = build_store(tree, Some(cache));
        store.upgrade_cache();
        store.delete(&vec![3]);

        // ---
        let result_range = store.range(range.clone()).collect::<BTreeMap<_, _>>();

        // ---

        let expected_range = [(vec![4_u8], vec![44_u8]), (vec![5], vec![55])]
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

    #[test]
    fn range_work_for_persist_and_cached_values_without_deleted_with_block() {
        let mut tree = build_tree();

        for (key, value) in [(1, 11), (2, 22), (3, 33), (4, 44), (5, 55), (6, 66)] {
            tree.set(vec![key], vec![value]);
        }

        let mut cache = KVCache::default();

        cache.storage.insert(vec![1], vec![111]);
        cache.delete.insert(vec![2]);
        cache.delete.insert(vec![5]);

        let range = ..vec![6];

        let mut store = build_store(tree, Some(cache));
        store.upgrade_cache();

        store.set(vec![1], vec![1]);
        store.set(vec![3], vec![3]);
        store.set(vec![5], vec![55]);
        store.delete(&vec![4]);

        // ---
        let result_range = store.range(range.clone()).collect::<BTreeMap<_, _>>();

        // ---

        let expected_range = [
            (vec![1_u8], vec![1_u8]),
            (vec![3], vec![3]),
            (vec![5], vec![55]),
        ]
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
            tx: cache.unwrap_or_default(),
            _marker: PhantomData,
            block: Default::default(),
        }
    }

    fn build_other_store<T>() -> KVBank<MemDB, T> {
        KVBank {
            persistent: Arc::new(RwLock::new(build_tree())),
            tx: Default::default(),
            _marker: PhantomData,
            block: Default::default(),
        }
    }
}
