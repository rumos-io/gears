use database::{prefix::PrefixDB, Database};
use kv_store::{
    types::{
        kv::{immutable::KVStore, mutable::KVStoreMut},
        multi::MultiBank,
    },
    ApplicationStore, StoreKey,
};
use tendermint::types::proto::event::Event;

use crate::types::store::kv::{mutable::StoreMut, Store};

use super::{ImmutableContext, MutableContext, QueryableContext, TransactionalContext};

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
    fn height(&self) -> u64 {
        unreachable!("inner type that is not supposed to provide external interfaces")
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key)).into()
    }
}

impl<DB: Database, SK: StoreKey> ImmutableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }
}

impl<DB: Database, SK: StoreKey> MutableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }

    fn events_drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    fn get_time(&self) -> Option<tendermint::types::time::Timestamp> {
        unreachable!("inner type that is not supposed to provide external interfaces")
    }

    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key)).into()
    }
}
