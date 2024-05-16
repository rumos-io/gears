use std::collections::{BTreeMap, HashSet};

/// Storage for store cache
#[derive(Debug, Clone, Default)]
pub(crate) struct KVCache {
    pub(crate) storage: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) delete: HashSet<Vec<u8>>,
}

impl KVCache {
    /// Take out all cache from storages.
    pub fn take(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        (
            std::mem::take(&mut self.storage),
            std::mem::take(&mut self.delete),
        )
    }

    /// Get value from cache
    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<&Vec<u8>> {
        if self.delete.contains(k.as_ref()) {
            return None;
        }

        self.storage.get(k.as_ref())
    }

    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        let _ = self.delete.insert(k.to_owned());
        self.storage.remove(k)
    }

    pub fn set<KI: IntoIterator<Item = u8>, VI: IntoIterator<Item = u8>>(
        &mut self,
        key: KI,
        value: VI,
    ) {
        let key: Vec<u8> = key.into_iter().collect();

        let _ = self.delete.remove(&key);
        self.storage.insert(key, value.into_iter().collect());
    }
}

#[derive(Debug)]
pub struct CacheCommitData<SK>(pub(crate) Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)>);

impl<SK> CacheCommitData<SK> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

impl<SK> IntoIterator for CacheCommitData<SK> {
    type Item = (SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
