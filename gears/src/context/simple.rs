use database::{prefix::PrefixDB, Database};
use kv_store::{
    types::{kv::immutable::KVStore, multi::MultiBank},
    ApplicationStore, StoreKey, TransactionStore,
};

use crate::types::store::kv::Store;

use super::{InfallibleContext, QueryableContext};

#[derive(Debug)]
pub enum SimpleBackend<'a, DB, SK> {
    Application(&'a mut MultiBank<DB, SK, ApplicationStore>),
    Transactional(&'a mut MultiBank<DB, SK, TransactionStore>),
}

impl<'a, DB, SK> From<&'a mut MultiBank<DB, SK, ApplicationStore>> for SimpleBackend<'a, DB, SK> {
    fn from(value: &'a mut MultiBank<DB, SK, ApplicationStore>) -> Self {
        Self::Application(value)
    }
}

impl<'a, DB, SK> From<&'a mut MultiBank<DB, SK, TransactionStore>> for SimpleBackend<'a, DB, SK> {
    fn from(value: &'a mut MultiBank<DB, SK, TransactionStore>) -> Self {
        Self::Transactional(value)
    }
}

#[derive(Debug)]
pub struct SimpleContext<'a, DB, SK> {
    multi_store: SimpleBackend<'a, DB, SK>,
    height: u64,
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {
    pub fn new(multi_store: SimpleBackend<'a, DB, SK>, height: u64) -> Self {
        Self {
            multi_store,
            height,
        }
    }
}

impl<'a, DB, SK> SimpleContext<'a, DB, SK> {}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for SimpleContext<'_, DB, SK> {
    fn height(&self) -> u64 {
        self.height
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        match &self.multi_store {
            SimpleBackend::Application(var) => KVStore::from(var.kv_store(store_key)).into(),
            SimpleBackend::Transactional(var) => KVStore::from(var.kv_store(store_key)).into(),
        }
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
