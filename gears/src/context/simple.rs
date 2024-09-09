use database::{prefix::PrefixDB, Database};
use kv_store::{
    bank::multi::{ApplicationMultiBank, TransactionMultiBank},
    store::kv::immutable::KVStore,
    StoreKey,
};
use tendermint::types::chain_id::ChainId;

use crate::types::store::kv::Store;

use super::{InfallibleContext, QueryableContext};

#[derive(Debug)]
pub enum SimpleBackend<'a, DB, SK> {
    Application(&'a mut ApplicationMultiBank<DB, SK>),
    Transactional(&'a mut TransactionMultiBank<DB, SK>),
}

impl<'a, DB, SK> From<&'a mut ApplicationMultiBank<DB, SK>> for SimpleBackend<'a, DB, SK> {
    fn from(value: &'a mut ApplicationMultiBank<DB, SK>) -> Self {
        Self::Application(value)
    }
}

impl<'a, DB, SK> From<&'a mut TransactionMultiBank<DB, SK>> for SimpleBackend<'a, DB, SK> {
    fn from(value: &'a mut TransactionMultiBank<DB, SK>) -> Self {
        Self::Transactional(value)
    }
}

#[derive(Debug)]
pub struct SimpleContext<'a, DB, SK> {
    multi_store: SimpleBackend<'a, DB, SK>,
    height: u32,
    chain_id: ChainId,
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {
    pub fn new(multi_store: SimpleBackend<'a, DB, SK>, height: u32, chain_id: ChainId) -> Self {
        Self {
            multi_store,
            height,
            chain_id,
        }
    }
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn height(&self) -> u32 {
        self.height
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        match &self.multi_store {
            SimpleBackend::Application(var) => KVStore::from(var.kv_store(store_key)).into(),
            SimpleBackend::Transactional(var) => KVStore::from(var.kv_store(store_key)).into(),
        }
    }

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        match &self.multi_store {
            SimpleBackend::Application(var) => KVStore::from(var.kv_store(store_key)),
            SimpleBackend::Transactional(var) => KVStore::from(var.kv_store(store_key)),
        }
    }
}
