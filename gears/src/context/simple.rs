use database::{prefix::PrefixDB, Database};
use kv_store::{
    types::{
        kv::{immutable::KVStore, mutable::KVStoreMut},
        multi::MultiBank,
    },
    ApplicationStore, StoreKey, TransactionStore,
    types::{kv::immutable::KVStore, multi::MultiBank},
    ApplicationStore, StoreKey,
};
use tendermint::types::proto::event::Event;

use crate::types::store::kv::Store;

use super::{InfallibleContext, QueryableContext};

#[derive(Debug)]
enum SimpleBackend<'a, DB, SK> {
    Application(&'a mut MultiBank<DB, SK, ApplicationStore>),
    Transactional(&'a mut MultiBank<DB, SK, TransactionStore>),
}

#[derive(Debug)]
pub struct SimpleContext<'a, DB, SK> {
    multi_store: SimpleBackend<'a, DB, SK>,
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


impl<'a, DB, SK> From<&'a mut MultiBank<DB, SK, ApplicationStore>> for SimpleContext<'a, DB, SK> {
    fn from(value: &'a mut MultiBank<DB, SK, ApplicationStore>) -> Self {
        Self {
            multi_store: SimpleBackend::Application(value),
            events: Vec::new(),
            height,
        }
    }
}

impl<'a, DB, SK> From<&'a mut MultiBank<DB, SK, TransactionStore>> for SimpleContext<'a, DB, SK> {
    fn from(value: &'a mut MultiBank<DB, SK, TransactionStore>) -> Self {
        Self {
            multi_store: SimpleBackend::Transactional(value),
            events: Vec::new(),
        }
    }
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        match &self.multi_store {
            SimpleBackend::Application(var) => KVStore::from(var.kv_store(store_key)).into(),
            SimpleBackend::Transactional(var) => KVStore::from(var.kv_store(store_key)).into(),
        }
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match &self.multi_store {
            SimpleBackend::Application(var) => KVStore::from(var.kv_store(store_key)),
            SimpleBackend::Transactional(var) => KVStore::from(var.kv_store(store_key)),
        }
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContextMut<DB, SK> for SimpleContext<'_, DB, SK> {
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        match &mut self.multi_store {
            SimpleBackend::Application(var) => KVStoreMut::from(var.kv_store_mut(store_key)),
            SimpleBackend::Transactional(var) => KVStoreMut::from(var.kv_store_mut(store_key)),
        }
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

    fn get_time(&self) -> Timestamp {
        unreachable!("inner type that is not supposed to provide external interfaces")
    }

    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>> {
        match &mut self.multi_store {
            SimpleBackend::Application(var) => KVStoreMut::from(var.kv_store_mut(store_key)).into(),
            SimpleBackend::Transactional(var) => {
                KVStoreMut::from(var.kv_store_mut(store_key)).into()
            }
        }
    }
}
