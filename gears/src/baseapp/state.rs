use database::Database;
use kv_store::{types::multi::MultiBank, ApplicationStore};

use crate::{
    application::handlers::node::ABCIHandler,
    types::gas::{basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, Gas, GasMeter},
};

use super::mode::{check::CheckTxMode, deliver::DeliverTxMode};

#[derive(Debug)]
pub struct ApplicationState<DB, AH: ABCIHandler> {
    pub(super) check_mode: CheckTxMode<DB, AH>,
    pub(super) deliver_mode: DeliverTxMode<DB, AH>,
}

impl<DB: Database, AH: ABCIHandler> ApplicationState<DB, AH> {
    pub fn new(max_gas: Gas, global_ms: &MultiBank<DB, AH::StoreKey, ApplicationStore>) -> Self {
        Self {
            check_mode: CheckTxMode::new(max_gas, global_ms.to_cache_kind()),
            deliver_mode: DeliverTxMode::new(max_gas, global_ms.to_cache_kind()),
        }
    }

    pub fn replace_meter(&mut self, max_gas: Gas) {
        match max_gas {
            Gas::Infinite => {
                self.check_mode.block_gas_meter = GasMeter::new(Box::<InfiniteGasMeter>::default());
                self.deliver_mode.block_gas_meter =
                    GasMeter::new(Box::<InfiniteGasMeter>::default());
            }
            Gas::Finite(max_gas) => {
                self.check_mode.block_gas_meter =
                    GasMeter::new(Box::new(BasicGasMeter::new(max_gas)));
                self.deliver_mode.block_gas_meter =
                    GasMeter::new(Box::new(BasicGasMeter::new(max_gas)));
            }
        }
    }

    pub fn cache_clear(&mut self) {
        self.check_mode.multi_store.caches_clear();
        self.deliver_mode.multi_store.caches_clear();
    }

    pub fn cache_update(&mut self, store: &mut MultiBank<DB, AH::StoreKey, ApplicationStore>) {
        let cache = store.caches_copy();

        self.check_mode.multi_store.caches_update(cache.clone());
        self.deliver_mode.multi_store.caches_update(cache);
    }
}
