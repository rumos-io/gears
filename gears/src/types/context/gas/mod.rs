pub mod descriptor;
pub mod kind;
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::types::gas::{Gas, GasErrors, GasMeter};

use self::{descriptor::MeterDescriptor, kind::MeterKind};

/// Wrapper around any gas meter which prevents usage of gas over limit with type system
#[derive(Debug, Clone)]
pub struct CtxGasMeter<DS> {
    meter: Arc<RwLock<Box<dyn GasMeter>>>,
    _descriptor: PhantomData<DS>,
}

impl<DS> CtxGasMeter<DS> {
    pub fn new(meter: Arc<RwLock<Box<dyn GasMeter>>>) -> Self {
        Self {
            meter,
            _descriptor: PhantomData,
        }
    }
}

impl<DS: MeterKind> CtxGasMeter<DS> {
    pub fn consume_to_limit<MD: MeterDescriptor>(&mut self) -> Result<(), GasErrors> {
        let mut lock = self.meter.write().expect("poisoned lock");

        let gas = lock.gas_consumed_to_limit();
        lock.consume_gas(gas, MD::name().to_owned())
    }

    pub fn consume_gas<MD: MeterDescriptor>(&mut self, amount: Gas) -> Result<(), GasErrors> {
        self.meter
            .write()
            .expect("poisoned lock")
            .consume_gas(amount, MD::name().to_owned())
    }

    pub fn is_out_of_gas(&self) -> bool {
        self.meter.read().expect("poisoned lock").is_out_of_gas()
    }
}

