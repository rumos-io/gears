use std::collections::HashMap;

pub enum StoreKey {
    Bank,
    Auth,
    AuthParams,
}

#[derive(Debug, Clone)]
pub struct MultiStore {
    bank_store: KVStore,
    auth_store: KVStore,
    auth_params_store: KVStore,
}

impl MultiStore {
    pub fn new() -> Self {
        MultiStore {
            bank_store: KVStore::new(),
            auth_store: KVStore::new(),
            auth_params_store: KVStore::new(),
        }
    }

    pub fn get_kv_store(&self, store_key: StoreKey) -> &KVStore {
        match store_key {
            StoreKey::Bank => &self.bank_store,
            StoreKey::Auth => &self.auth_store,
            StoreKey::AuthParams => &self.auth_params_store,
        }
    }

    pub fn get_mutable_kv_store(&mut self, store_key: StoreKey) -> &mut KVStore {
        match store_key {
            StoreKey::Bank => &mut self.bank_store,
            StoreKey::Auth => &mut self.auth_store,
            StoreKey::AuthParams => &mut self.auth_params_store,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KVStore {
    core: HashMap<Vec<u8>, Vec<u8>>,
}

impl KVStore {
    pub fn new() -> Self {
        KVStore {
            core: HashMap::new(),
        }
    }

    pub fn get(&self, k: &[u8]) -> Option<&Vec<u8>> {
        self.core.get(k)
    }

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
        self.core.insert(k, v)
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

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
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
