use std::{collections::HashMap, sync::Arc};

use database::{Database, PrefixDB};
use trees::iavl::{CacheSize, QueryTree, Tree};

use crate::{error::StoreError, StoreKey, TREE_CACHE_SIZE};

use self::{kv::QueryKVStoreV2, versioned::VersionedQueryMultiStore};

pub mod kv;
pub mod versioned;

pub struct QueryMultiStoreV2<DB, SK>(HashMap<SK, Tree<PrefixDB<DB>>>);

impl<DB: Database, SK: StoreKey> QueryMultiStoreV2<DB, SK> {
    pub fn new(db: Arc<DB>) -> Result<Self, StoreError> {
        let mut stores = HashMap::new();

        for key in SK::iter() {
            let prefix = key.name().as_bytes().to_vec();

            let tree = Tree::new(
                PrefixDB::new(Arc::clone(&db), prefix),
                None,
                CacheSize::try_from(TREE_CACHE_SIZE).expect("Unreachable. Tree cache size is > 0"),
            )?;

            stores.insert(key, tree);
        }

        Ok(Self(stores))
    }

    pub fn to_versioned(
        &self,
        version: u32,
    ) -> Result<VersionedQueryMultiStore<'_, DB, SK>, StoreError> {
        let mut stores = HashMap::with_capacity(self.0.len());

        for (sk, tree) in &self.0 {
            let query_store = QueryKVStoreV2::new(QueryTree::new(tree, version)?);
            stores.insert(sk, query_store);
        }

        Ok(VersionedQueryMultiStore(stores))
    }
}
