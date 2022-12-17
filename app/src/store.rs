use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

#[derive(Debug, Clone)]
pub struct Store {
    core: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    prefix: Vec<u8>,
}

//TODO: fix TODOs

impl Store {
    pub fn new() -> Self {
        let core = HashMap::new();
        return Store {
            core: Arc::new(RwLock::new(core)),
            prefix: vec![],
        };
    }

    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let mut full_key = self.prefix.clone();
        full_key.extend(k);
        self.core
            .read()
            .expect("Mutex will not be poisoned")
            .get(&full_key)
            .map(Clone::clone)
    }

    pub fn set(&self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
        self.core
            .write()
            .expect("Mutex will not be poisoned")
            .insert(k, v)
    }

    pub fn get_state_hash() -> Vec<u8> {
        return vec![];
    }

    pub fn get_sub_store(&self, mut prefix: Vec<u8>) -> Self {
        let mut full_prefix = self.prefix.clone();
        full_prefix.append(&mut prefix);
        return Store {
            core: self.core.clone(),
            prefix,
        };
    }
}

impl IntoIterator for Store {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;

    fn into_iter(self) -> Self::IntoIter {
        let prefix = self.prefix.clone();
        let prefix2 = self.prefix.clone();
        let iter = self
            .core
            .read()
            .expect("Mutex will not be poisoned")
            .clone()
            .into_iter()
            .filter(move |x| {
                let key = &x.0;
                let key_prefix = &key[0..prefix.len()];
                let eq = key_prefix == &prefix[..];
                return eq;
            })
            .map(move |x| (x.0[prefix2.len()..].to_vec(), x.1));

        return Box::new(iter);
    }
}
