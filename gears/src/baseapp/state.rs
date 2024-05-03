use std::sync::{Arc, RwLock};

use store_crate::{database::Database, StoreKey};

use crate::types::gas::{
    basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, Gas, GasMeter,
};

use super::mode::{check::CheckTxMode, deliver::DeliverTxMode};

#[derive(Debug)]
pub struct ApplicationState<DB, SK> {
    pub(super) check_mode: CheckTxMode<DB, SK>,
    pub(super) deliver_mode: DeliverTxMode<DB, SK>,
}

impl<DB: Database, SK: StoreKey> ApplicationState<DB, SK> {
    pub fn new(db: Arc<DB>, max_gas: Gas) -> Self {
        Self {
            check_mode: CheckTxMode::new(Arc::clone(&db), max_gas),
            deliver_mode: DeliverTxMode::new(db, max_gas),
        }
    }

    pub fn new_sync(db: Arc<DB>, max_gas: Gas) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(db, max_gas)))
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
}
