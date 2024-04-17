use std::fmt::Debug;

use store_crate::database::{Database, PrefixDB};
use store_crate::{
    types::{kv::KVStore, multi::MultiStore},
    QueryableMultiKVStore, StoreKey, TransactionalMultiKVStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

use crate::types::header::Header;

use super::{QueryableContext, TransactionalContext};

#[derive(Debug, former::Former)]
pub struct TxContext<'a, DB, SK> {
    multi_store: &'a mut MultiStore<DB, SK>,
    pub height: u64,
    pub events: Vec<Event>,
    pub header: Header,
    #[alias(tx)]
    _tx_bytes: Vec<u8>,
}

impl<'a, DB: Database, SK: StoreKey> TxContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut MultiStore<DB, SK>,
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
}

impl<DB: Database, SK: StoreKey> QueryableContext<PrefixDB<DB>, SK> for TxContext<'_, DB, SK> {
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

impl<DB: Database, SK: StoreKey> TransactionalContext<PrefixDB<DB>, SK> for TxContext<'_, DB, SK> {
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

// #[derive(Debug, former::Former)]
// pub struct RunTxContext<'a, DB, SK, GM> {
//     pub multi_store: &'a mut MultiStore<DB, SK>,
//     pub height: u64,
//     pub events: Vec<Event>,
//     pub header: Header,
//     pub block_gas_meter: CtxGasMeter<GM>,
// }

// impl<'a, DB: Database, SK: StoreKey, GM: GasMeter> RunTxContext<'a, DB, SK, GM> {
//     pub fn gas_meter(&self) -> &CtxGasMeter<GM> {
//         &self.block_gas_meter
//     }

//     pub fn gas_meter_mut(&mut self) -> &mut CtxGasMeter<GM> {
//         &mut self.block_gas_meter
//     }
// }

// impl<DB: Database, SK: StoreKey, GM> QueryableContext<PrefixDB<DB>, SK>
//     for RunTxContext<'_, DB, SK, GM>
// {
//     type KVStore = KVStore<PrefixDB<DB>>;

//     fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
//         self.multi_store.kv_store(store_key)
//     }

//     fn height(&self) -> u64 {
//         self.height
//     }

//     fn chain_id(&self) -> &ChainId {
//         &self.header.chain_id
//     }
// }

// impl<DB: Database, SK: StoreKey, GM> TransactionalContext<PrefixDB<DB>, SK>
//     for RunTxContext<'_, DB, SK, GM>
// {
//     type KVStoreMut = KVStore<PrefixDB<DB>>;

//     fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut {
//         self.multi_store.kv_store_mut(store_key)
//     }

//     fn push_event(&mut self, event: Event) {
//         self.events.push(event);
//     }

//     fn append_events(&mut self, mut events: Vec<Event>) {
//         self.events.append(&mut events);
//     }
// }
