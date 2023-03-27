use std::{
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use database::{PrefixDB, DB};
use trees::iavl::{Range, Tree};

use crate::error::AppError;

use super::hash::{self, StoreInfo};

pub enum Store {
    Bank,
    Auth,
    Params,
}

//TODO:
// 1. this overlaps with Auth::Module
// 2. use strum crate to iterate over stores
// 3. move prefix store into separate file
impl Store {
    pub fn name(&self) -> &str {
        match self {
            Store::Bank => "bank",
            Store::Auth => "acc",
            Store::Params => "params",
        }
    }
}

#[derive(Debug)]
pub struct MultiStore<T: DB> {
    bank_store: KVStore<PrefixDB<T>>,
    auth_store: KVStore<PrefixDB<T>>,
    params_store: KVStore<PrefixDB<T>>,
    head_version: u32,
    head_commit_hash: [u8; 32],
}

impl<T: DB> MultiStore<T> {
    pub fn new(db: T) -> Self {
        let db = Arc::new(db);

        let bank_store = KVStore::new(
            PrefixDB::new(db.clone(), Store::Bank.name().as_bytes().to_vec()),
            None,
        )
        .unwrap();
        let auth_store = KVStore::new(
            PrefixDB::new(db.clone(), Store::Auth.name().as_bytes().to_vec()),
            None,
        )
        .unwrap();
        let params_store = KVStore::new(
            PrefixDB::new(db, Store::Params.name().as_bytes().to_vec()),
            None,
        )
        .unwrap();

        let bank_info = StoreInfo {
            name: Store::Bank.name().into(),
            hash: bank_store.head_commit_hash(),
        };

        let auth_info = StoreInfo {
            name: Store::Auth.name().into(),
            hash: auth_store.head_commit_hash(),
        };

        let params_info = StoreInfo {
            name: Store::Params.name().into(),
            hash: params_store.head_commit_hash(),
        };

        let store_infos = [bank_info, auth_info, params_info].into();

        MultiStore {
            head_version: bank_store.last_committed_version(),
            bank_store,
            auth_store,
            params_store,
            head_commit_hash: hash::hash_store_infos(store_infos),
        }
    }

    pub fn get_head_version(&self) -> u32 {
        self.head_version
    }

    pub fn get_head_commit_hash(&self) -> [u8; 32] {
        self.head_commit_hash
    }

    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        match store_key {
            Store::Bank => &self.bank_store,
            Store::Auth => &self.auth_store,
            Store::Params => &self.params_store,
        }
    }

    pub fn get_mutable_kv_store(&mut self, store_key: Store) -> &mut KVStore<PrefixDB<T>> {
        match store_key {
            Store::Bank => &mut self.bank_store,
            Store::Auth => &mut self.auth_store,
            Store::Params => &mut self.params_store,
        }
    }

    /// Writes then clears each store's tx cache to the store's block cache then clears the tx caches
    pub fn write_then_clear_tx_caches(&mut self) {
        self.bank_store.write_then_clear_tx_cache();
        self.auth_store.write_then_clear_tx_cache();
        self.params_store.write_then_clear_tx_cache();
    }

    /// Clears the tx caches
    pub fn clear_tx_caches(&mut self) {
        self.bank_store.clear_tx_cache();
        self.auth_store.clear_tx_cache();
        self.params_store.clear_tx_cache();
    }

    pub fn commit(&mut self) -> [u8; 32] {
        let bank_info = StoreInfo {
            name: Store::Bank.name().into(),
            hash: self.bank_store.commit(),
        };

        let auth_info = StoreInfo {
            name: Store::Auth.name().into(),
            hash: self.auth_store.commit(),
        };

        let params_info = StoreInfo {
            name: Store::Params.name().into(),
            hash: self.params_store.commit(),
        };

        let store_infos = [bank_info, auth_info, params_info].into();
        let hash = hash::hash_store_infos(store_infos);

        self.head_commit_hash = hash;
        self.head_version += 1;
        hash
    }
}

#[derive(Debug)]
pub struct KVStore<T: DB> {
    persistent_store: Tree<T>,
    block_cache: BTreeMap<Vec<u8>, Vec<u8>>,
    tx_cache: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl<T: DB> KVStore<T> {
    pub fn new(db: T, target_version: Option<u32>) -> Result<Self, AppError> {
        Ok(KVStore {
            persistent_store: Tree::new(db, target_version)?,
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

    pub fn get_immutable_prefix_store(&self, prefix: Vec<u8>) -> ImmutablePrefixStore<T> {
        ImmutablePrefixStore {
            store: self,
            prefix,
        }
    }

    pub fn get_mutable_prefix_store(&mut self, prefix: Vec<u8>) -> MutablePrefixStore<T> {
        MutablePrefixStore {
            store: self,
            prefix,
        }
    }

    pub fn range<R>(&self, range: R) -> Range<R, T>
    where
        R: RangeBounds<Vec<u8>>,
    {
        //TODO: this doesn't iterate over cached values
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

/// Wraps an immutable reference to a KVStore with a prefix
pub struct ImmutablePrefixStore<'a, T: DB> {
    store: &'a KVStore<T>,
    prefix: Vec<u8>,
}

impl<'a, T: DB> ImmutablePrefixStore<'a, T> {
    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let full_key = [&self.prefix, k].concat();
        self.store.get(&full_key)
    }

    pub fn range<R: RangeBounds<Vec<u8>>>(&self, range: R) -> PrefixRange<'a, T> {
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

pub struct PrefixRange<'a, T: DB> {
    parent_range: Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), T>,
    prefix_length: usize,
}

impl<'a, T: DB> Iterator for PrefixRange<'a, T> {
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
pub struct MutablePrefixStore<'a, T: DB> {
    store: &'a mut KVStore<T>,
    prefix: Vec<u8>,
}

impl<'a, T: DB> MutablePrefixStore<'a, T> {
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
}
