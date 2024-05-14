use std::collections::{BTreeMap, HashSet};

use database::Database;

use crate::{CacheKind, StoreKey};

use super::MultiBank;

impl<DB: Database, SK: StoreKey> MultiBank<DB, SK, CacheKind> {
    pub fn commit(&mut self) -> CacheCommitData<SK> {
        let mut map: Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)> =
            Vec::with_capacity(self.stores.len());

        for (sk, store) in &mut self.stores {
            let (set, delete) = store.commit();
            map.push((sk.to_owned(), set, delete));
        }

        CacheCommitData(map)
    }

    pub fn head_version_set(&mut self, version: u32) {
        self.head_version = version;
    }

    pub fn head_commit_hash_set(&mut self, hash: [u8; 32]) {
        self.head_commit_hash = hash;
    }
}

pub struct CacheCommitData<SK>(Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)>);

impl<SK> IntoIterator for CacheCommitData<SK> {
    type Item = (SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
