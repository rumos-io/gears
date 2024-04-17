pub mod mode;
pub mod mode_impl;
mod unconsumed_impl;

use store_crate::{
    database::{Database, PrefixDB},
    types::{kv::KVStore, multi::MultiStore},
    QueryableMultiKVStore, StoreKey, TransactionalMultiKVStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

use crate::types::header::Header;

use super::{
    gas::{BlockDescriptor, CtxGasMeter, UnConsumed},
    QueryableContext, TransactionalContext,
};

#[derive(Debug)]
pub struct TxContextWithGas<'a, DB, SK, ST> {
    pub events: Vec<Event>,

    multi_store: &'a mut MultiStore<DB, SK>,
    height: u64,
    header: Header,
    block_gas_meter: CtxGasMeter<ST, BlockDescriptor>,
}

impl<'a, DB, SK> TxContextWithGas<'a, DB, SK, UnConsumed> {
    pub fn new(
        multi_store: &'a mut MultiStore<DB, SK>,
        height: u64,
        header: Header,
        block_gas_meter: CtxGasMeter<UnConsumed, BlockDescriptor>,
    ) -> Self {
        Self {
            events: Vec::new(),
            multi_store,
            height,
            header,
            block_gas_meter,
        }
    }
}

impl<DB: Database, SK: StoreKey, ST> QueryableContext<PrefixDB<DB>, SK>
    for TxContextWithGas<'_, DB, SK, ST>
{
    type KVStore = KVStore<PrefixDB<DB>>;
    type MultiStore = MultiStore<DB, SK>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
    }

    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }

    fn multi_store(&self) -> &Self::MultiStore {
        self.multi_store
    }
}

impl<DB: Database, SK: StoreKey, ST> TransactionalContext<PrefixDB<DB>, SK>
    for TxContextWithGas<'_, DB, SK, ST>
{
    type KVStoreMut = KVStore<PrefixDB<DB>>;
    type MultiStoreMut = MultiStore<DB, SK>;

    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut {
        self.multi_store.kv_store_mut(store_key)
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

    fn multi_store_mut(&mut self) -> &mut Self::MultiStoreMut {
        self.multi_store
    }
}
