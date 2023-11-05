use crate::types::context::context::Context;
use database::{Database, PrefixDB};
use store_crate::{KVStore, MultiStore, StoreKey};
use tendermint_informal::{abci::Event, block::Header};

pub struct TxContext<'a, T: Database, SK: StoreKey> {
    multi_store: &'a mut MultiStore<T, SK>,
    pub height: u64,
    pub events: Vec<Event>,
    pub header: Header,
    _tx_bytes: Vec<u8>,
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
            _tx_bytes: tx_bytes,
        }
    }

    pub fn get_header(&self) -> &Header {
        &self.header
    }

    pub fn as_any<'b>(&'b mut self) -> Context<'b, 'a, T, SK> {
        Context::TxContext(self)
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
        return self.multi_store.get_mutable_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}
