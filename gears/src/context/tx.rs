use std::{cell::RefCell, sync::Arc};

use database::{prefix::PrefixDB, Database};
use kv::{
    bank::multi::TransactionMultiBank,
    store::multi::{immutable::MultiStore, mutable::MultiStoreMut},
    StoreKey,
};
use tendermint::types::{
    chain_id::ChainId,
    proto::{event::Event, header::Header},
    time::Timestamp,
};

use crate::{
    baseapp::{options::NodeOptions, ConsensusParams},
    types::{
        gas::{
            kind::{BlockKind, TxKind},
            GasMeter,
        },
        store::{
            gas::{
                guard::GasGuard,
                kv::{mutable::GasKVStoreMut, GasKVStore},
            },
            kv::{mutable::StoreMut, Store},
        },
    },
};

use super::{QueryableContext, TransactionalContext};

#[derive(Debug)]
pub struct TxContext<'a, DB, SK> {
    pub gas_meter: Arc<RefCell<GasMeter<TxKind>>>,
    pub events: Vec<Event>,
    pub options: NodeOptions,
    pub(crate) height: u32,
    pub(crate) header: Header,
    pub(crate) block_gas_meter: &'a mut GasMeter<BlockKind>,
    pub(crate) consensus_params: ConsensusParams,
    multi_store: &'a mut TransactionMultiBank<DB, SK>,
    is_check: bool,
}

impl<'a, DB, SK> TxContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut TransactionMultiBank<DB, SK>,
        height: u32,
        header: Header,
        consensus_params: ConsensusParams,
        gas_meter: GasMeter<TxKind>,
        block_gas_meter: &'a mut GasMeter<BlockKind>,
        is_check: bool,
        options: NodeOptions,
    ) -> Self {
        Self {
            events: Vec::new(),
            multi_store,
            height,
            header,
            gas_meter: Arc::new(RefCell::new(gas_meter)),
            block_gas_meter,
            consensus_params,
            is_check,
            options,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn multi_store(&self) -> MultiStore<'_, DB, SK> {
        MultiStore::from(&*self.multi_store)
    }

    pub(crate) fn multi_store_mut(&mut self) -> MultiStoreMut<'_, DB, SK> {
        MultiStoreMut::from(&mut *self.multi_store)
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

impl<DB: Database, SK: StoreKey> TxContext<'_, DB, SK> {
    pub fn consensus_params(&self) -> &ConsensusParams {
        &self.consensus_params
    }

    #[inline]
    pub fn is_check(&self) -> bool {
        self.is_check
    }

    pub fn kv_store(&self, store_key: &SK) -> GasKVStore<'_, PrefixDB<DB>> {
        GasKVStore::new(
            GasGuard::new(Arc::clone(&self.gas_meter)),
            self.multi_store.kv_store(store_key).into(),
        )
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> GasKVStoreMut<'_, PrefixDB<DB>> {
        GasKVStoreMut::new(
            GasGuard::new(Arc::clone(&self.gas_meter)),
            self.multi_store.kv_store_mut(store_key).into(),
        )
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for TxContext<'_, DB, SK> {
    fn height(&self) -> u32 {
        self.height
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        Store::from(self.kv_store(store_key))
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<DB, SK> for TxContext<'_, DB, SK> {
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
        self.header.time.clone()
    }

    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>> {
        StoreMut::from(self.kv_store_mut(store_key))
    }
}
