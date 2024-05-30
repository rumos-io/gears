use std::{cell::RefCell, sync::Arc};

use database::{prefix::PrefixDB, Database};
use kv_store::{
    types::{
        kv::{immutable::KVStore, mutable::KVStoreMut, store_cache::CacheCommitList},
        multi::{immutable::MultiStore, mutable::MultiStoreMut, MultiBank},
    },
    StoreKey, TransactionStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event, time::Timestamp};

use crate::{
    baseapp::{options::NodeOptions, ConsensusParams},
    types::{
        gas::{
            kind::{BlockKind, TxKind},
            GasMeter,
        },
        header::Header,
        store::gas::{
            guard::GasGuard,
            kv::{mutable::GasKVStoreMut, GasKVStore},
        },
    },
};

use super::{ImmutableGasContext, MutableGasContext, QueryableContext, TransactionalContext};

#[derive(Debug)]
pub struct TxContext<'a, DB, SK> {
    pub gas_meter: Arc<RefCell<GasMeter<TxKind>>>,
    pub events: Vec<Event>,
    pub options: NodeOptions,
    pub(crate) height: u64,
    pub(crate) header: Header,
    pub(crate) block_gas_meter: &'a mut GasMeter<BlockKind>,
    pub(crate) consensus_params: ConsensusParams,
    multi_store: &'a mut MultiBank<DB, SK, TransactionStore>,
    is_check: bool,
}

impl<'a, DB, SK> TxContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut MultiBank<DB, SK, TransactionStore>,
        height: u64,
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
}

impl<DB: Database, SK: StoreKey> TxContext<'_, DB, SK> {
    pub(crate) fn commit(&mut self) -> CacheCommitList<SK> {
        self.multi_store.commit()
    }

    pub fn consensus_params(&self) -> &ConsensusParams {
        &self.consensus_params
    }

    #[inline]
    pub fn is_check(&self) -> bool {
        self.is_check
    }

    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for TxContext<'_, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }
}

impl<DB: Database, SK: StoreKey> ImmutableGasContext<DB, SK> for TxContext<'_, DB, SK> {
    fn kv_store(&self, store_key: &SK) -> GasKVStore<'_, PrefixDB<DB>> {
        GasKVStore::new(
            Some(GasGuard::new(Arc::clone(&self.gas_meter))),
            self.kv_store(store_key),
        )
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

    fn get_time(&self) -> Option<Timestamp> {
        self.header.time.clone()
    }
}

impl<DB: Database, SK: StoreKey> MutableGasContext<DB, SK> for TxContext<'_, DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> GasKVStoreMut<'_, PrefixDB<DB>> {
        GasKVStoreMut::new(
            Some(GasGuard::new(Arc::clone(&self.gas_meter))),
            self.kv_store_mut(store_key),
        )
    }
}
