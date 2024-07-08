use std::sync::{Arc, RwLock};

use database::Database;
use trees::iavl::Tree;

use crate::{error::POISONED_LOCK, types::kv::store_cache::KVCache};

#[derive(Debug)]
pub struct TransactionKVBank<DB> {
    pub(crate) persistent: Arc<RwLock<Tree<DB>>>,
    pub(crate) tx: KVCache,
    pub(crate) block: KVCache,
}

impl<DB: Database> TransactionKVBank<DB> {
    /// Read persistent database
    #[inline]
    pub fn persistent(&self) -> std::sync::RwLockReadGuard<Tree<DB>> {
        self.persistent.read().expect(POISONED_LOCK)
    }

    /// Clear uncommitted cache for tx
    #[inline]
    pub fn tx_cache_clear(&mut self) {
        self.tx.storage.clear();
        self.tx.delete.clear();
    }

    /// Clear uncommitted cache for block
    #[inline]
    pub fn block_cache_clear(&mut self) {
        self.block.storage.clear();
        self.block.delete.clear();
    }

    /// Upgrade cache means push changes from tx to block
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

    /// Delete value from storage
    #[inline]
    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.tx
            .delete(k)
            .or(self.persistent.read().expect(POISONED_LOCK).get(k))
    }

    /// Set or append value
    #[inline]
    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        self.tx.set(key, value)
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        match self.tx.get(k.as_ref()).ok()? {
            Some(var) => Some(var.to_owned()),
            None => self
                .block
                .get(k.as_ref())
                .ok()?
                .cloned()
                .or_else(|| self.persistent.read().expect(POISONED_LOCK).get(k.as_ref())),
        }
    }

    // pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, DB> {
    //     let cached_values = self
    //         .block
    //         .storage
    //         .range(range.clone())
    //         .chain(self.tx.storage.range(range.clone()))
    //         .map(|(first, second)| (Cow::Borrowed(first), Cow::Borrowed(second)))
    //         .collect::<HashMap<_, _>>();

    //     let tree = self.persistent.read().expect(POISONED_LOCK);
    //     let persisted_values = tree
    //         .range(range)
    //         .filter(|(key, _)| {
    //             !(self.tx.delete.contains(&**key)
    //                 || (self.block.delete.contains(&**key)
    //                     && !self.tx.storage.contains_key(&**key)))
    //         })
    //         .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second)));

    //     MergedRange::merge(cached_values.into_iter(), persisted_values).into()
    // }
}

#[cfg(test)]
mod tests {

    use database::MemDB;

    use crate::TREE_CACHE_SIZE;

    use super::*;

    #[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TestStore;

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

    fn build_store(tree: Tree<MemDB>, cache: Option<KVCache>) -> TransactionKVBank<MemDB> {
        TransactionKVBank {
            persistent: Arc::new(RwLock::new(tree)),
            tx: cache.unwrap_or_default(),
            block: Default::default(),
        }
    }
}
