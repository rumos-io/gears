use database::Database;
use kv::bank::multi::ApplicationMultiBank;

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
    pub fn new(max_gas: Gas, global_ms: &ApplicationMultiBank<DB, AH::StoreKey>) -> Self {
        Self {
            check_mode: CheckTxMode::new(max_gas, global_ms.to_tx_kind()),
            deliver_mode: DeliverTxMode::new(max_gas, global_ms.to_tx_kind()),
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

    pub fn multi_store_replace(&mut self, store: &mut ApplicationMultiBank<DB, AH::StoreKey>) {
        self.check_mode.multi_store = store.to_tx_kind();
        self.deliver_mode.multi_store = store.to_tx_kind();
    }

    pub fn push_changes(&mut self, app_ms: &mut ApplicationMultiBank<DB, AH::StoreKey>) {
        self.check_mode.multi_store.tx_cache_clear();
        self.check_mode.multi_store.block_cache_clear();

        app_ms.consume_tx_cache(&mut self.deliver_mode.multi_store);
    }
}
