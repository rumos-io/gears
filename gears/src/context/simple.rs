use database::{prefix::PrefixDB, Database};
use kv_store::{
    types::{kv::immutable::KVStore, multi::MultiBank},
    ApplicationStore, StoreKey,
};
use tendermint::types::proto::event::Event;

use crate::types::store::kv::Store;

use super::{InfallibleContext, QueryableContext};

#[derive(Debug)]
pub struct SimpleContext<'a, DB, SK> {
    multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>,
    height: u64,
    pub events: Vec<Event>,
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {
    pub fn new(multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>, height: u64) -> Self {
        Self {
            multi_store,
            events: Vec::new(),
            height,
        }
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key)).into()
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }
}
