use database::{prefix::PrefixDB, Database};
use store_crate::{
    types::{kv::immutable::KVStore, multi::MultiBank},
    ApplicationStore, StoreKey,
};
use tendermint::types::proto::event::Event;

use super::QueryableContext;

#[derive(Debug)]
pub struct SimpleContext<'a, DB, SK> {
    multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>,
    pub events: Vec<Event>,
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {
    pub fn new(multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>) -> Self {
        Self {
            multi_store,
            events: Vec::new(),
        }
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.multi_store.kv_store(store_key).into()
    }

    fn height(&self) -> u64 {
        //TODO: remove unreachable
        unreachable!("inner type that is not supposed to provide external interfaces")
    }
}
