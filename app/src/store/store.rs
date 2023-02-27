use std::collections::HashMap;

use trees::iavl::IAVLTree;

use crate::x::bank;

use super::hash::{self, StoreInfo};

pub enum Store {
    Bank,
    Auth,
    Params,
}

//TODO:
// 1. this overlaps with Auth::Module
// 2. use strum crate to iterate over stores
impl Store {
    pub fn name(&self) -> String {
        match self {
            Store::Bank => "bank".to_string(),
            Store::Auth => "acc".to_string(),
            Store::Params => "params".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiStore {
    bank_store: KVStore,
    auth_store: KVStore,
    params_store: KVStore,
}

impl MultiStore {
    pub fn new() -> Self {
        MultiStore {
            bank_store: KVStore::new(),
            auth_store: KVStore::new(),
            params_store: KVStore::new(),
        }
    }

    pub fn get_kv_store(&self, store_key: Store) -> &KVStore {
        match store_key {
            Store::Bank => &self.bank_store,
            Store::Auth => &self.auth_store,
            Store::Params => &self.params_store,
        }
    }

    pub fn get_mutable_kv_store(&mut self, store_key: Store) -> &mut KVStore {
        match store_key {
            Store::Bank => &mut self.bank_store,
            Store::Auth => &mut self.auth_store,
            Store::Params => &mut self.params_store,
        }
    }

    /// Write each store's tx cache to the store's block cache then clear's the tx caches
    pub fn write_then_clear_tx_caches(&mut self) {
        self.bank_store.write_then_clear_tx_cache();
        self.auth_store.write_then_clear_tx_cache();
        self.params_store.write_then_clear_tx_cache();
    }

    /// Write each store's tx cache to the store's block cache then clear's the tx caches
    pub fn clear_tx_caches(&mut self) {
        self.bank_store.clear_tx_cache();
        self.auth_store.clear_tx_cache();
        self.params_store.clear_tx_cache();
    }

    pub fn commit(&mut self) -> [u8; 32] {
        let bank_info = StoreInfo {
            name: Store::Bank.name(),
            hash: self.bank_store.commit(),
        };

        let auth_info = StoreInfo {
            name: Store::Auth.name(),
            hash: self.auth_store.commit(),
        };

        let params_info = StoreInfo {
            name: Store::Params.name(),
            hash: self.params_store.commit(),
        };

        let store_infos = [bank_info, auth_info, params_info].into();
        hash::hash_store_infos(store_infos)
    }
}

#[derive(Debug, Clone)]
pub struct KVStore {
    tree_store: IAVLTree,
    block_cache: HashMap<Vec<u8>, Vec<u8>>,
    tx_cache: HashMap<Vec<u8>, Vec<u8>>,
}

impl KVStore {
    pub fn new() -> Self {
        KVStore {
            tree_store: IAVLTree::new(),
            block_cache: HashMap::new(),
            tx_cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        let tx_cache_val = self.tx_cache.get(key);

        if tx_cache_val.is_none() {
            let block_cache_val = self.block_cache.get(key);

            if block_cache_val.is_none() {
                return self.tree_store.get(key);
            };

            return block_cache_val;
        }

        tx_cache_val
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        if key.is_empty() {
            // TODO: copied from SDK, need to understand why this is needed and maybe create a type which captures the restriction
            panic!("key is empty")
        }

        self.tx_cache.insert(key, value);
    }

    pub fn get_immutable_prefix_store(&self, prefix: Vec<u8>) -> ImmutablePrefixStore {
        ImmutablePrefixStore {
            store: self,
            prefix,
        }
    }

    pub fn get_mutable_prefix_store(&mut self, prefix: Vec<u8>) -> MutablePrefixStore {
        MutablePrefixStore {
            store: self,
            prefix,
        }
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
            self.tree_store.set(key.to_owned(), value.to_owned())
        }
        self.block_cache.clear();
    }

    pub fn commit(&mut self) -> [u8; 32] {
        self.write_then_clear_block_cache();
        let (hash, _) = self.tree_store.save_version();
        hash
    }
}

/// Wraps an immutable reference to a KVStore with a prefix
pub struct ImmutablePrefixStore<'a> {
    store: &'a KVStore,
    prefix: Vec<u8>,
}

impl<'a> ImmutablePrefixStore<'a> {
    pub fn get(&self, k: &[u8]) -> Option<&Vec<u8>> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        self.store.get(&full_key)
    }

    pub fn get_prefix_store(&self, mut prefix: Vec<u8>) -> ImmutablePrefixStore {
        let mut full_prefix = self.prefix.clone();
        full_prefix.append(&mut prefix);

        ImmutablePrefixStore {
            store: self.store,
            prefix: full_prefix,
        }
    }
}

/// Wraps an mutable reference to a KVStore with a prefix
pub struct MutablePrefixStore<'a> {
    store: &'a mut KVStore,
    prefix: Vec<u8>,
}

impl<'a> MutablePrefixStore<'a> {
    pub fn get(&self, k: &[u8]) -> Option<&Vec<u8>> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        self.store.get(&full_key)
    }

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) {
        let full_key = self.get_full_key(&k);
        self.store.set(full_key, v)
    }

    pub fn get_prefix_store(&mut self, mut prefix: Vec<u8>) -> MutablePrefixStore {
        let mut full_prefix = self.prefix.clone();
        full_prefix.append(&mut prefix);

        MutablePrefixStore {
            store: self.store,
            prefix: full_prefix,
        }
    }

    fn get_full_key(&self, k: &[u8]) -> Vec<u8> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        return full_key;
    }

    pub fn _get_prefix(&self) -> Vec<u8> {
        return self.prefix.clone();
    }
}

impl<'a> IntoIterator for ImmutablePrefixStore<'a> {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;

    fn into_iter(self) -> Self::IntoIter {
        //TODO: this doesn't iterate over cached values
        let prefix = self.prefix.clone();
        let prefix2 = self.prefix.clone();
        let iter = self
            .store
            .tree_store
            .clone()
            .into_iter()
            .filter(move |x| {
                let key = &x.0;
                if key.len() < prefix.len() {
                    return false;
                }
                let key_prefix = &key[0..prefix.len()];
                return key_prefix == &prefix[..];
            })
            .map(move |x| (x.0[prefix2.len()..].to_vec(), x.1));

        return Box::new(iter);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn prefix_store_iterator_works() {
        let mut store = KVStore::new();
        store.set(vec![0, 1], vec![1]);
        store.set(vec![1, 3], vec![2]);

        let prefix_store = store.get_immutable_prefix_store(vec![1]);

        for (k, v) in prefix_store {
            assert_eq!(k, vec![3]);
            assert_eq!(v, vec![2]);
        }
    }
}
