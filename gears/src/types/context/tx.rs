use store_crate::{
    database::{Database, PrefixDB},
    types::{
        kv::{mutable::KVStoreMut, KVStore},
        multi::MultiStore,
    },
    QueryableMultiKVStore, StoreKey, TransactionalMultiKVStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

use crate::types::{
    gas::{kind::TxKind, GasMeter},
    header::Header,
};

use super::{KVContext, QueryableContext, TransactionalContext};

#[derive(Debug)]
pub struct TxContext<'a, DB, SK> {
    pub gas_meter: GasMeter<TxKind>,
    pub(crate) events: Vec<Event>,
    multi_store: &'a mut MultiStore<DB, SK>,
    height: u64,
    header: Header,
}

impl<'a, DB, SK> TxContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut MultiStore<DB, SK>,
        height: u64,
        header: Header,
        gas_meter: GasMeter<TxKind>,
    ) -> Self {
        Self {
            events: Vec::new(),
            multi_store,
            height,
            header,
            gas_meter,
        }
    }

    // pub(crate) fn multi_store(&self) -> &MultiStore<DB, SK> {
    //     self.multi_store
    // }

    pub(crate) fn multi_store_mut(&mut self) -> &mut MultiStore<DB, SK> {
        self.multi_store
    }
}

impl<'a, DB: Database, SK: StoreKey> QueryableContext<PrefixDB<DB>, SK> for TxContext<'a, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }
}

impl<DB: Database, SK: StoreKey> KVContext<PrefixDB<DB>, SK> for TxContext<'_, DB, SK> {
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.multi_store.kv_store(store_key).into()
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<PrefixDB<DB>, SK> for TxContext<'_, DB, SK> {
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
