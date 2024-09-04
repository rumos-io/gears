use database::prefix::PrefixDB;
use database::Database;
use kv_store::{
    bank::multi::ApplicationMultiBank,
    store::kv::{immutable::KVStore, mutable::KVStoreMut},
    StoreKey,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event, time::timestamp::Timestamp};

use crate::types::store::kv::Store;
use crate::{baseapp::ConsensusParams, types::store::kv::mutable::StoreMut};

use super::{InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext};

#[derive(Debug)]
pub struct InitContext<'a, DB, SK> {
    multi_store: &'a mut ApplicationMultiBank<DB, SK>,
    consensus_params: ConsensusParams,
    pub(crate) height: u32,
    pub(crate) time: Timestamp,
    pub events: Vec<Event>,
    pub(crate) chain_id: ChainId,
}

impl<'a, DB, SK> InitContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut ApplicationMultiBank<DB, SK>,
        height: u32,
        time: Timestamp,
        chain_id: ChainId,
        consensus_params: ConsensusParams,
    ) -> Self {
        InitContext {
            multi_store,
            height,
            time,
            events: Vec::new(),
            chain_id,
            consensus_params,
        }
    }
}

impl<'a, DB: Database, SK: StoreKey> InitContext<'a, DB, SK> {
    pub fn consensus_params(&self) -> &ConsensusParams {
        &self.consensus_params
    }

    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for InitContext<'_, DB, SK> {
    fn height(&self) -> u32 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        Store::from(self.kv_store(store_key))
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContext<DB, SK> for InitContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.kv_store(store_key)
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContextMut<DB, SK> for InitContext<'_, DB, SK> {
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        self.kv_store_mut(store_key)
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<DB, SK> for InitContext<'_, DB, SK> {
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
        self.time
    }

    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>> {
        StoreMut::from(self.kv_store_mut(store_key))
    }
}
