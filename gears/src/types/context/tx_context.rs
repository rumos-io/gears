use crate::{
    place_holder::EventManager,
    types::{
        context::context::Priority,
        gas::{gas_meter::GasMeter, infinite_meter::InfiniteGasMeter},
    },
};
use bytes::Bytes;
use database::{Database, PrefixDB};
use store_crate::{place_holders::CacheMS, KVStore, MultiStore, StoreKey};
use tendermint_informal::{abci::Event, block::Header};

use super::context::ContextTrait;

pub struct TxContext<'a, T: Database, SK: StoreKey> {
    multi_store: &'a mut MultiStore<T, SK>,
    pub height: u64,
    pub events: Vec<Event>,
    pub header: Header,
    pub priority: Priority,
    _tx_bytes: Vec<u8>,
    gas_meter: InfiniteGasMeter,       //TODO: Trait
    block_gas_meter: InfiniteGasMeter, //TODO: Trait
}

impl<'a, T: Database, SK: StoreKey> TxContext<'a, T, SK> {
    pub fn new(
        multi_store: &'a mut MultiStore<T, SK>,
        height: u64,
        header: Header,
        tx_bytes: Vec<u8>,
    ) -> Self {
        TxContext {
            multi_store,
            height,
            events: vec![],
            header,
            priority: Priority(0),
            _tx_bytes: tx_bytes,
            gas_meter: InfiniteGasMeter::new(),
            block_gas_meter: InfiniteGasMeter::new(),
        }
    }

    pub fn get_header(&self) -> &Header {
        &self.header
    }

    pub fn cache_tx_context(&self, _tx_bytes: &Bytes) -> (TxContext<'_, T, SK>, CacheMS) {
        let ms_cache = self.multi_store.cache_multi_store();

        if ms_cache.is_tracing_enabled() {
            ms_cache.tracing_context_set();
        }

        todo!()
    }

    pub fn event_manager_set(&mut self, manager: EventManager) {
        todo!()
    }
}

impl<'a, T: Database, SK: StoreKey> ContextTrait<T, SK> for TxContext<'a, T, SK> {
    fn gas_meter(&self) -> &dyn GasMeter {
        &self.gas_meter
    }

    fn block_gas_meter(&self) -> &dyn GasMeter {
        &self.block_gas_meter
    }

    fn gas_meter_mut(&mut self) -> &mut dyn GasMeter {
        &mut self.gas_meter
    }

    fn block_gas_meter_mut(&mut self) -> &mut dyn GasMeter {
        &mut self.block_gas_meter
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
        self.multi_store.get_kv_store(store_key)
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
        self.multi_store.get_mutable_kv_store(store_key)
    }

    fn multi_store_mut(&mut self) -> &mut MultiStore<T, SK> {
        &mut self.multi_store
    }

    fn get_height(&self) -> u64 {
        self.height
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}
