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
    core: IAVLTree,
    cache: HashMap<Vec<u8>, Vec<u8>>,
}

impl KVStore {
    pub fn new() -> Self {
        KVStore {
            core: IAVLTree::new(),
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        let cache_val = self.cache.get(key);

        if cache_val.is_none() {
            return self.core.get(key);
        }

        cache_val
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.set_cache_value(key, value);
    }

    fn set_cache_value(&mut self, key: Vec<u8>, value: Vec<u8>) {
        if key.is_empty() {
            // TODO: copied from SDK, need to understand why this is needed and maybe create a type which captures the restriction
            panic!("key is empty")
        }

        self.cache.insert(key, value);
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

    fn write(&mut self) {
        let mut keys: Vec<&Vec<u8>> = self.cache.keys().collect();
        keys.sort();

        for key in keys {
            let value = self
                .cache
                .get(key)
                .expect("key is definitely in the HashMap");
            self.core.set(key.to_owned(), value.to_owned())
        }
        self.cache.clear();
    }

    pub fn commit(&mut self) -> [u8; 32] {
        self.write();
        let (hash, _) = self.core.save_version();
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
            .core
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
