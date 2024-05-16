use std::collections::{BTreeMap, HashSet};

use database::Database;

use crate::{StoreKey, TransactionStore};

use super::MultiBank;

impl<DB: Database, SK: StoreKey> MultiBank<DB, SK, TransactionStore> {
    pub fn commit(&mut self) -> CacheCommitData<SK> {
        let mut map: Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)> =
            Vec::with_capacity(self.stores.len());

        for (sk, store) in &mut self.stores {
            let (set, delete) = store.commit();
            map.push((sk.to_owned(), set, delete));
        }

        CacheCommitData(map)
    }
}

#[derive(Debug)]
pub struct CacheCommitData<SK>(Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)>);

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
