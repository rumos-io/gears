use std::sync::{Arc, RwLock};

use crate::types::gas::{
    basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, Gas, GasMeter,
};

use super::mode::{check::CheckTxMode, deliver::DeliverTxMode};

#[derive(Debug)]
pub struct ApplicationState {
    pub(super) check_mode: CheckTxMode,
    pub(super) deliver_mode: DeliverTxMode,
}

impl ApplicationState {
    pub fn new(max_gas: Gas) -> Self {
        Self {
            check_mode: CheckTxMode::new(max_gas),
            deliver_mode: DeliverTxMode::new(max_gas),
        }
    }

    pub fn new_sync(max_gas: Gas) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(max_gas)))
    }

    pub fn replace_meter(&mut self, max_gas: Gas) {
        match max_gas > Gas::new(0) {
            true => {
                self.check_mode.block_gas_meter =
                    GasMeter::new(Box::new(BasicGasMeter::new(max_gas)));
                self.deliver_mode.block_gas_meter =
                    GasMeter::new(Box::new(BasicGasMeter::new(max_gas)));
            }
            false => {
                self.check_mode.block_gas_meter =
                    GasMeter::new(Box::new(InfiniteGasMeter::default()));
                self.deliver_mode.block_gas_meter =
                    GasMeter::new(Box::new(InfiniteGasMeter::default()));
            }
        }
    }
}
