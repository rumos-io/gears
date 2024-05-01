use store_crate::database::{Database, PrefixDB};
use store_crate::types::kv::mutable::KVStoreMut;
use store_crate::{
    types::{kv::KVStore, multi::MultiStore},
    QueryableMultiKVStore, StoreKey, TransactionalMultiKVStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

use super::{QueryableContext, TransactionalContext};

#[derive(Debug)]
pub struct InitContext<'a, DB, SK> {
    multi_store: &'a mut MultiStore<DB, SK>,
    pub height: u64,
    pub events: Vec<Event>,
    pub chain_id: ChainId,
}

impl<'a, DB, SK> InitContext<'a, DB, SK> {
    pub fn new(multi_store: &'a mut MultiStore<DB, SK>, height: u64, chain_id: ChainId) -> Self {
        InitContext {
            multi_store,
            height,
            events: vec![],
            chain_id,
        }
    }
}

impl<'a, DB: Database, SK: StoreKey> QueryableContext<PrefixDB<DB>, SK>
    for InitContext<'a, DB, SK>
{
    type MultiStore = MultiStore<DB, SK>;

    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.multi_store.kv_store(store_key).into()
    }

    fn multi_store(&self) -> &Self::MultiStore {
        self.multi_store
    }

    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<PrefixDB<DB>, SK>
    for InitContext<'_, DB, SK>
{
    type MultiStoreMut = MultiStore<DB, SK>;

    fn multi_store_mut(&mut self) -> &mut Self::MultiStoreMut {
        self.multi_store
    }

    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        self.multi_store.kv_store_mut(store_key).into()
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }

    fn events_drain(&mut self) -> Vec<Event> {
        self.events.drain(..).collect()
    }
}
