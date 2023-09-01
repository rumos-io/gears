use std::{
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use database::{Database, PrefixDB};
use std::{collections::HashMap, hash::Hash};
use strum::IntoEnumIterator;
use trees::iavl::{Range, Tree};

use crate::{error::Error, QueryKVStore};

use super::hash::{self, StoreInfo};

const TREE_CACHE_SIZE: usize = 100_000;

//TODO:
// 1. move prefix store into separate file
// 2. remove unwraps

#[derive(Debug)]
pub struct MultiStore<DB: Database, SK: StoreKey> {
    pub(crate) head_version: u32,
    pub(crate) head_commit_hash: [u8; 32],
    pub(crate) stores: HashMap<SK, KVStore<PrefixDB<DB>>>,
}

pub trait StoreKey: Hash + Eq + IntoEnumIterator + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

impl<DB: Database, SK: StoreKey> MultiStore<DB, SK> {
    pub fn new(db: DB) -> Self {
        let db = Arc::new(db);
        let mut store_infos = vec![];
        let mut stores = HashMap::new();
        let mut head_version = 0;

        for store in SK::iter() {
            // TODO: check that store names are not prefixes
            let prefix = store.name().as_bytes().to_vec();
            let kv_store = KVStore::new(PrefixDB::new(db.clone(), prefix), None).unwrap();

            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.head_commit_hash(),
            };

            head_version = kv_store.last_committed_version();

            stores.insert(store, kv_store);
            store_infos.push(store_info)
        }

        MultiStore {
            head_version,
            head_commit_hash: hash::hash_store_infos(store_infos),
            stores,
        }
    }

    pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<DB>> {
        self.stores
            .get(store_key)
            .expect("a store for every key is guaranteed to exist")
    }

    pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<DB>> {
        self.stores
            .get_mut(store_key)
            .expect("a store for every key is guaranteed to exist")
    }

    pub fn get_head_version(&self) -> u32 {
        self.head_version
    }

    pub fn get_head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }

    /// Writes then clears each store's tx cache to the store's block cache then clears the tx caches
    pub fn write_then_clear_tx_caches(&mut self) {
        for (_, store) in &mut self.stores {
            store.write_then_clear_tx_cache();
        }
    }

    /// Clears the tx caches
    pub fn clear_tx_caches(&mut self) {
        for (_, store) in &mut self.stores {
            store.clear_tx_cache();
        }
    }

    pub fn commit(&mut self) -> [u8; 32] {
        let mut store_infos = vec![];
        for (store, kv_store) in &mut self.stores {
            let store_info = StoreInfo {
                name: store.name().into(),
                hash: kv_store.commit(),
            };

            store_infos.push(store_info)
        }

        let hash = hash::hash_store_infos(store_infos);

        self.head_commit_hash = hash;
        self.head_version += 1;
        hash
    }
}

#[derive(Debug)]
pub struct KVStore<DB: Database> {
    pub(crate) persistent_store: Tree<DB>,
    block_cache: BTreeMap<Vec<u8>, Vec<u8>>,
    tx_cache: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl<DB: Database> KVStore<DB> {
    pub fn new(db: DB, target_version: Option<u32>) -> Result<Self, Error> {
        Ok(KVStore {
            persistent_store: Tree::new(db, target_version, TREE_CACHE_SIZE)?,
            block_cache: BTreeMap::new(),
            tx_cache: BTreeMap::new(),
        })
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let tx_cache_val = self.tx_cache.get(key);

        if tx_cache_val.is_none() {
            let block_cache_val = self.block_cache.get(key);

            if block_cache_val.is_none() {
                return self.persistent_store.get(key);
            };

            return block_cache_val.cloned();
        }

        tx_cache_val.cloned()
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        if key.is_empty() {
            // TODO: copied from SDK, need to understand why this is needed and maybe create a type which captures the restriction
            panic!("key is empty")
        }

        self.tx_cache.insert(key, value);
    }

    pub fn get_immutable_prefix_store(&self, prefix: Vec<u8>) -> ImmutablePrefixStore<DB> {
        ImmutablePrefixStore {
            store: self.into(),
            prefix,
        }
    }

    pub fn get_mutable_prefix_store(&mut self, prefix: Vec<u8>) -> MutablePrefixStore<DB> {
        MutablePrefixStore {
            store: self,
            prefix,
        }
    }

    pub fn range<R>(&self, range: R) -> Range<R, DB>
    where
        R: RangeBounds<Vec<u8>> + Clone,
    {
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

    pub fn commit(&mut self) -> [u8; 32] {
        self.write_then_clear_tx_cache();
        self.write_then_clear_block_cache();
        let (hash, _) = self.persistent_store.save_version().unwrap(); //TODO: is it safe to assume this won't ever error?
        hash
    }

    pub fn head_commit_hash(&self) -> [u8; 32] {
        self.persistent_store.root_hash()
    }

    pub fn last_committed_version(&self) -> u32 {
        self.persistent_store.loaded_version()
    }
}

pub(crate) enum AnyKVStore<'a, DB: Database> {
    KVStore(&'a KVStore<DB>),
    QueryKVStore(&'a QueryKVStore<'a, DB>),
}

impl<'a, DB: Database> From<&'a KVStore<DB>> for AnyKVStore<'a, DB> {
    fn from(kv_store: &'a KVStore<DB>) -> Self {
        Self::KVStore(kv_store)
    }
}

impl<'a, DB: Database> From<&'a QueryKVStore<'a, DB>> for AnyKVStore<'a, DB> {
    fn from(kv_store: &'a QueryKVStore<'a, DB>) -> Self {
        Self::QueryKVStore(kv_store)
    }
}

impl<'a, DB: Database> AnyKVStore<'a, DB> {
    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        match self {
            AnyKVStore::KVStore(store) => store.get(k),
            AnyKVStore::QueryKVStore(store) => store.get(k),
        }
    }

    pub fn range<R>(&self, range: R) -> Range<R, DB>
    where
        R: RangeBounds<Vec<u8>> + Clone,
    {
        match self {
            AnyKVStore::KVStore(store) => store.range(range),
            AnyKVStore::QueryKVStore(store) => store.range(range),
        }
    }
}

/// Wraps an immutable reference to a KVStore with a prefix
pub struct ImmutablePrefixStore<'a, DB: Database> {
    pub(crate) store: AnyKVStore<'a, DB>,
    pub(crate) prefix: Vec<u8>,
}

impl<'a, DB: Database> ImmutablePrefixStore<'a, DB> {
    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k].concat();
        self.store.get(&full_key)
    }

    pub fn range<R: RangeBounds<Vec<u8>>>(&'a self, range: R) -> PrefixRange<'a, DB> {
        let new_start = match range.start_bound() {
            Bound::Included(b) => Bound::Included([self.prefix.clone(), b.clone()].concat()),
            Bound::Excluded(b) => Bound::Excluded([self.prefix.clone(), b.clone()].concat()),
            Bound::Unbounded => Bound::Included(self.prefix.clone()),
        };

        let new_end = match range.end_bound() {
            Bound::Included(b) => Bound::Included([self.prefix.clone(), b.clone()].concat()),
            Bound::Excluded(b) => Bound::Excluded([self.prefix.clone(), b.clone()].concat()),
            Bound::Unbounded => prefix_end_bound(self.prefix.clone()),
        };

        PrefixRange {
            parent_range: self.store.range((new_start, new_end)),
            prefix_length: self.prefix.len(),
        }
    }
}

pub struct PrefixRange<'a, DB: Database> {
    parent_range: Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>,
    prefix_length: usize,
}

impl<'a, DB: Database> Iterator for PrefixRange<'a, DB> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.parent_range.next()?;

        // Remove the prefix from the key - this is safe since all returned keys will include the prefix.
        // TODO: what if the key now has zero length, is this safe given the check on KVStore set.
        let truncated_key = next.0[self.prefix_length..].to_vec();

        Some((truncated_key, next.1))
    }
}

/// Returns the KVStore Bound that would end an unbounded upper
/// range query on a PrefixStore with the given prefix
///
/// That is the smallest x such that, prefix + y < x for all y. If
/// no such x exists (i.e. prefix = vec![255; N]; for some N) it returns Bound::Unbounded
fn prefix_end_bound(mut prefix: Vec<u8>) -> Bound<Vec<u8>> {
    loop {
        let last = prefix.last_mut();

        match last {
            None => return Bound::Unbounded,
            Some(last) => {
                if *last != 255 {
                    *last += 1;
                    return Bound::Excluded(prefix);
                }
                prefix.pop();
            }
        }
    }
}

/// Wraps an mutable reference to a KVStore with a prefix
pub struct MutablePrefixStore<'a, DB: Database> {
    store: &'a mut KVStore<DB>,
    prefix: Vec<u8>,
}

impl<'a, DB: Database> MutablePrefixStore<'a, DB> {
    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k].concat();
        self.store.get(&full_key)
    }

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) {
        // TODO: do we need to check for zero length keys as with the KVStore::set?
        let full_key = [self.prefix.clone(), k].concat();
        self.store.set(full_key, v)
    }
}

// TODO: the tests on iterators don't enforce the ordering? i.e.
// assert!(expected_pairs.iter().all(|e| {
//     let cmp = (e.0.clone(), e.1.clone());
//     got_pairs.contains(&cmp)
// }));

#[cfg(test)]
mod tests {

    use database::MemDB;

    use super::*;

    #[test]
    fn prefix_store_range_works() {
        let db = MemDB::new();
        let mut store = KVStore::new(db, None).unwrap();
        store.set(vec![0], vec![1]);
        store.set(vec![0, 1], vec![2]);
        store.set(vec![0, 2], vec![3]);
        store.set(vec![1], vec![4]);
        store.set(vec![1, 1], vec![5]);
        store.set(vec![1, 2], vec![6]);
        store.set(vec![1, 3], vec![7]);
        store.set(vec![1, 4], vec![8]);
        store.set(vec![1, 5], vec![9]);
        store.set(vec![2], vec![10]);
        store.set(vec![2, 1], vec![11]);
        store.set(vec![2, 2], vec![12]);
        store.set(vec![2, 3], vec![13]);
        store.commit(); //TODO: this won't be needed once the KVStore iterator correctly incorporates cached values

        let prefix_store = store.get_immutable_prefix_store(vec![1]);

        // unbounded
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = prefix_store.range(..).collect();
        let expected_pairs = vec![
            (vec![], vec![4]),
            (vec![1], vec![5]),
            (vec![2], vec![6]),
            (vec![3], vec![7]),
            (vec![4], vec![8]),
            (vec![5], vec![9]),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (e.0.clone(), e.1.clone());
            got_pairs.contains(&cmp)
        }));

        // [,]
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = prefix_store.range(vec![1]..=vec![3]).collect();
        let expected_pairs = vec![(vec![1], vec![5]), (vec![2], vec![6]), (vec![3], vec![7])];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (e.0.clone(), e.1.clone());
            got_pairs.contains(&cmp)
        }));

        // (,)
        let start = vec![1];
        let stop = vec![3];
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = prefix_store
            .range((Bound::Excluded(start), Bound::Excluded(stop)))
            .collect();
        let expected_pairs = vec![(vec![2], vec![6])];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (e.0.clone(), e.1.clone());
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn prefix_end_bound_works() {
        let prefix = vec![1, 2, 3];
        let expected = vec![1, 2, 4];

        assert!(matches!(
            prefix_end_bound(prefix),
            Bound::Excluded(x) if x == expected));

        let prefix = vec![1, 2, 255];
        let expected = vec![1, 3];

        assert!(matches!(
            prefix_end_bound(prefix),
            Bound::Excluded(x) if x == expected));

        let prefix = vec![1, 255, 255];
        let expected = vec![2];

        assert!(matches!(
            prefix_end_bound(prefix),
            Bound::Excluded(x) if x == expected));

        let prefix = vec![255, 255, 255];

        assert!(matches!(prefix_end_bound(prefix), Bound::Unbounded));
    }

    // TODO: uncomment this test once merged range works
    // /// Tests whether kv range works with cached and persisted values
    // #[test]
    // fn kv_store_merged_range_works() {
    //     let db = MemDB::new();
    //     let mut store = KVStore::new(db, None).unwrap();

    //     // values in this group will be in the persistent store
    //     store.set(vec![1], vec![1]);
    //     store.set(vec![7], vec![13]); // shadowed by value in tx cache
    //     store.set(vec![10], vec![2]); // shadowed by value in block cache
    //     store.set(vec![14], vec![234]); // shadowed by value in block cache and tx cache
    //     store.commit();

    //     // values in this group will be in the block cache
    //     store.set(vec![2], vec![3]);
    //     store.set(vec![9], vec![4]); // shadowed by value in tx cache
    //     store.set(vec![10], vec![7]); // shadows a persisted value
    //     store.set(vec![14], vec![212]); // shadows a persisted value AND shadowed by value in tx cache
    //     store.write_then_clear_tx_cache();

    //     // values in this group will be in the tx cache
    //     store.set(vec![3], vec![5]);
    //     store.set(vec![8], vec![6]);
    //     store.set(vec![7], vec![5]); // shadows a persisted value
    //     store.set(vec![9], vec![6]); // shadows a block cache value
    //     store.set(vec![14], vec![212]); // shadows a persisted value which shadows a persisted value

    //     let start = vec![0];
    //     let stop = vec![20];
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = store
    //         .range((Bound::Excluded(start), Bound::Excluded(stop)))
    //         .collect();
    //     let expected_pairs = vec![
    //         (vec![1], vec![1]),
    //         (vec![2], vec![3]),
    //         (vec![3], vec![5]),
    //         (vec![7], vec![5]),
    //         (vec![8], vec![6]),
    //         (vec![9], vec![6]),
    //         (vec![10], vec![7]),
    //         (vec![14], vec![212]),
    //     ];

    //     assert_eq!(expected_pairs, got_pairs);
    // }
}
