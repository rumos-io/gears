use database::{prefix::PrefixDB, Database};
use kv_store::{
    types::{
        kv::{immutable::KVStore, mutable::KVStoreMut},
        multi::MultiBank,
    },
    ApplicationStore, StoreKey,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event, time::Timestamp};

use crate::types::{
    header::Header,
    store::kv::{mutable::StoreMut, Store},
};

use super::{
    ImmutableContext, ImmutableGasContext, MutableContext, MutableGasContext, QueryableContext,
    TransactionalContext,
};

#[derive(Debug)]
pub struct BlockContext<'a, DB, SK> {
    multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>,
    pub(crate) height: u64,
    pub header: Header,
    pub events: Vec<Event>,
}

impl<'a, DB, SK> BlockContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>,
        height: u64,
        header: Header,
    ) -> Self {
        BlockContext {
            multi_store,
            height,
            events: Vec::new(),
            header,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }
}

impl<DB: Database, SK: StoreKey> BlockContext<'_, DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }
}

impl<DB: Database, SK: StoreKey> ImmutableContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.kv_store(store_key)
    }
}

impl<DB: Database, SK: StoreKey> MutableContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        self.kv_store_mut(store_key)
    }
}

impl<DB: Database, SK: StoreKey> ImmutableGasContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        Store::from(self.kv_store(store_key))
    }
}

impl<DB: Database, SK: StoreKey> MutableGasContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>> {
        StoreMut::from(self.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }

    fn events_drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    fn get_time(&self) -> Option<Timestamp> {
        self.header.time.clone()
    }
}
