use std::collections::{BTreeMap, HashSet};

use database::Database;

use crate::{types::kv::store_cache::CacheCommitList, StoreKey, TransactionStore};

use super::MultiBank;

impl<DB: Database, SK: StoreKey> MultiBank<DB, SK, TransactionStore> {
    pub fn commit(&mut self) -> CacheCommitList<SK> {
        let mut map: Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)> =
            Vec::with_capacity(self.stores.len());

        for (sk, store) in &mut self.stores {
            let (set, delete) = store.commit();
            map.push((sk.to_owned(), set, delete));
        }

        CacheCommitList(map)
    }

    pub fn into_commit(self) -> CacheCommitList<SK> {
        let mut map: Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)> =
            Vec::with_capacity(self.stores.len());

        for (sk, mut store) in self.stores {
            let (set, delete) = store.commit();
            map.push((sk, set, delete));
        }

        CacheCommitList(map)
    }
}
