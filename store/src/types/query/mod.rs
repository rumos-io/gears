use std::{collections::HashMap, marker::PhantomData};

use database::{Database, PrefixDB};
use trees::iavl::{QueryTree, Tree};

use crate::{error::StoreError, StoreKey};

use self::{kv::QueryKVStore, versioned::VersionedQueryMultiStore};

pub mod kv;
pub mod versioned;

#[derive(Debug)]
pub struct QueryMultiBank<'a, DB, SK> {
    tree: &'a Tree<PrefixDB<DB>>,
    _marker: PhantomData<SK>,
}

impl<'a, DB: Database, SK: StoreKey> QueryMultiBank<'a, DB, SK> {
    pub fn new(tree: &'a Tree<PrefixDB<DB>>) -> Self {
        Self {
            tree,
            _marker: PhantomData,
        }
    }

    pub fn to_versioned(
        &self,
        version: u32,
    ) -> Result<VersionedQueryMultiStore<'_, DB, SK>, StoreError> {
        let mut stores = HashMap::new();

        for key in SK::iter() {
            let query_store = QueryKVStore::new(QueryTree::new(self.tree, version)?);
            stores.insert(key, query_store);
        }

        Ok(VersionedQueryMultiStore(stores))
    }
}
