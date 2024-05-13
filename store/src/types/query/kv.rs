use std::ops::RangeBounds;

use database::Database;
use trees::iavl::{QueryTree, Tree};

use crate::{error::StoreError, types::prefix_v2::immutable::ImmutablePrefixStoreV2};

#[derive(Debug)]
pub struct QueryKVStore<'a, DB> {
    persistent_store: QueryTree<'a, DB>,
}

impl<DB: Database> QueryKVStore<'_, DB> {
    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(
        &self,
        range: R,
    ) -> crate::range::Range<'_, R, DB> {
        self.persistent_store.range(range).into()
    }

    pub fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.persistent_store.get(k.as_ref())
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        &self,
        prefix: I,
    ) -> ImmutablePrefixStoreV2<'_, DB> {
        ImmutablePrefixStoreV2 {
            store: self.into(),
            prefix: prefix.into_iter().collect(),
        }
    }
}

impl<'a, DB: Database> QueryKVStore<'a, DB> {
    pub fn new(persistent_store: &'a Tree<DB>, version: u32) -> Result<Self, StoreError> {
        Ok(QueryKVStore {
            persistent_store: QueryTree::new(persistent_store, version)?,
        })
    }
}

// #[cfg(test)]
// mod test {
//     use std::{borrow::Cow, ops::Bound};

//     use database::MemDB;

//     use crate::types::{kv::commit::CommitKVStore, query::kv::QueryKVStore};

//     #[test]
//     fn kv_store_merged_range_works() {
//         let db = MemDB::new();
//         let mut store = CommitKVStore::new(db, None).unwrap();

//         // values in this group will be in the persistent store
//         store.set(vec![1], vec![1]);
//         store.set(vec![7], vec![13]); // shadowed by value in tx cache
//         store.set(vec![10], vec![2]); // shadowed by value in block cache
//         store.set(vec![14], vec![234]); // shadowed by value in block cache and tx cache
//         store.commit();

//         // values in this group will be in the block cache
//         store.set(vec![2], vec![3]);
//         store.set(vec![9], vec![4]); // shadowed by value in tx cache
//         store.set(vec![10], vec![7]); // shadows a persisted value
//         store.set(vec![14], vec![212]); // shadows a persisted value AND shadowed by value in tx cache
//         store.cache.tx_upgrade_to_block();

//         // values in this group will be in the tx cache
//         store.set(vec![3], vec![5]);
//         store.set(vec![8], vec![6]);
//         store.set(vec![7], vec![5]); // shadows a persisted value
//         store.set(vec![9], vec![6]); // shadows a block cache value
//         store.set(vec![14], vec![212]); // shadows a persisted value which shadows a persisted value

//         let store =
//             QueryKVStore::new(&store.persistent_store, 0).expect("Failed to create QueryKVStore");

//         let start = vec![0];
//         let stop = vec![20];
//         let got_pairs = store
//             .range((Bound::Excluded(start), Bound::Excluded(stop)))
//             .collect::<Vec<_>>();
//         let expected_pairs = [
//             (vec![1], vec![1]),
//             (vec![7], vec![13]),
//             (vec![10], vec![2]),
//             (vec![14], vec![234]),
//         ]
//         .into_iter()
//         .map(|(first, second)| (Cow::Owned(first), Cow::Owned(second)))
//         .collect::<Vec<_>>();

//         // NOTE: For now - QueryTree iterate only over persisted values, expected to retrieve only them
//         assert_eq!(expected_pairs, got_pairs);
//     }
// }
