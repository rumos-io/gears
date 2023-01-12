use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MultiStore {
    core: HashMap<Vec<u8>, Vec<u8>>,
}

impl MultiStore {
    pub fn new() -> Self {
        MultiStore {
            core: HashMap::new(),
        }
    }

    pub fn get(&self, k: &[u8]) -> Option<&Vec<u8>> {
        self.core.get(k)
    }

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
        self.core.insert(k, v)
    }

    pub fn get_immutable_sub_store(&self, prefix: Vec<u8>) -> ImmutableSubStore {
        ImmutableSubStore {
            store: self,
            prefix,
        }
    }

    pub fn get_mutable_sub_store(&mut self, prefix: Vec<u8>) -> MutableSubStore {
        MutableSubStore {
            store: self,
            prefix,
        }
    }
}

/// Wraps an immutable reference to a MultiStore with a prefix
pub struct ImmutableSubStore<'a> {
    store: &'a MultiStore,
    prefix: Vec<u8>,
}

impl<'a> ImmutableSubStore<'a> {
    pub fn get(&self, k: &[u8]) -> Option<&Vec<u8>> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        self.store.get(&full_key)
    }

    pub fn get_sub_store(&self, mut prefix: Vec<u8>) -> ImmutableSubStore {
        let mut full_prefix = self.prefix.clone();
        full_prefix.append(&mut prefix);

        ImmutableSubStore {
            store: self.store,
            prefix: full_prefix,
        }
    }
}

/// Wraps an mutable reference to a MultiStore with a prefix
pub struct MutableSubStore<'a> {
    store: &'a mut MultiStore,
    prefix: Vec<u8>,
}

impl<'a> MutableSubStore<'a> {
    pub fn get(&self, k: &[u8]) -> Option<&Vec<u8>> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        self.store.get(&full_key)
    }

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
        let full_key = self.get_full_key(&k);
        self.store.set(full_key, v)
    }

    pub fn get_sub_store(&mut self, mut prefix: Vec<u8>) -> MutableSubStore {
        let mut full_prefix = self.prefix.clone();
        full_prefix.append(&mut prefix);

        MutableSubStore {
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

impl<'a> IntoIterator for ImmutableSubStore<'a> {
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
    fn sub_store_iterator_works() {
        let mut store = MultiStore::new();
        store.set(vec![0, 1], vec![1]);
        store.set(vec![1, 3], vec![2]);

        let sub_store = store.get_immutable_sub_store(vec![1]);

        for (k, v) in sub_store {
            assert_eq!(k, vec![3]);
            assert_eq!(v, vec![2]);
        }
    }
}
