use std::ops::RangeBounds;

use database::Database;
use trees::iavl::{QueryTree, Range};

use crate::{
    error::Error,
    types::{kv::KVStore, prefix::immutable::ImmutablePrefixStore},
    ReadKVStore,
};

pub struct QueryKVStore<'a, DB> {
    persistent_store: QueryTree<'a, DB>,
}

impl<'a, DB: Database> ReadKVStore<DB> for QueryKVStore<'a, DB> {
    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Option<Vec<u8>> {
        self.persistent_store.get(k.as_ref())
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(&self, prefix: I) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: self.into(),
            prefix: prefix.into_iter().collect(),
        }
    }

    fn range<R: RangeBounds<Vec<u8>> + Clone>(&self, range: R) -> Range<'_, R, DB> {
        self.persistent_store.range(range)
    }

    // fn get_keys(&self, key_prefix: &(impl AsRef<[u8]> + ?Sized)) -> Vec<Vec<u8>> {
    //     self.persistent_store
    //         .range(..)
    //         .map(|(key, _value)| key)
    //         .filter(|key| key.starts_with(key_prefix.as_ref()))
    //         .collect()
    // }
}

impl<'a, DB: Database> QueryKVStore<'a, DB> {
    pub fn new(kv_store: &'a KVStore<DB>, version: u32) -> Result<Self, Error> {
        Ok(QueryKVStore {
            persistent_store: QueryTree::new(&kv_store.persistent_store, version)?,
        })
    }
}
