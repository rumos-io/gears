use database::{prefix::PrefixDB, Database};

use crate::{bank::kv::transaction::TransactionKVBank, StoreKey};

use super::*;

#[derive(Debug)]
pub struct TransactionStore<DB, SK>(pub(crate) HashMap<SK, TransactionKVBank<PrefixDB<DB>>>);

impl<SK, DB> MultiBankBackend<DB, SK> for TransactionStore<DB, SK> {
    type Bank = TransactionKVBank<PrefixDB<DB>>;

    fn stores(&self) -> &HashMap<SK, Self::Bank> {
        &self.0
    }

    fn stores_mut(&mut self) -> &mut HashMap<SK, Self::Bank> {
        &mut self.0
    }
}

impl<DB: Database, SK: StoreKey> MultiBank<DB, SK, TransactionStore<DB, SK>> {
    pub fn tx_cache_clear(&mut self) {
        for store in self.backend.0.values_mut() {
            store.tx_cache_clear()
        }
    }

    pub fn upgrade_cache(&mut self) {
        for store in self.backend.0.values_mut() {
            store.upgrade_cache()
        }
    }
}
