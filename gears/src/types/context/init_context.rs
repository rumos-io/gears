use database::{Database, PrefixDB};
use store_crate::{
    types::{kv::KVStore, multi::MultiStore},
    ReadMultiKVStore, StoreKey, WriteMultiKVStore,
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

impl<DB: Database, SK: StoreKey> QueryableContext<PrefixDB<DB>, SK> for InitContext<'_, DB, SK> {
    type KVStore = KVStore<PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
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
    type KVStoreMut = KVStore<PrefixDB<DB>>;

    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut {
        self.multi_store.kv_store_mut(store_key)
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}
