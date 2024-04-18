use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::types::gas::{Gas, GasErrors, GasMeter};

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

impl<DS: Descriptor> CtxGasMeter<DS> {
    pub fn consume_to_limit(&mut self) -> Result<(), GasErrors> {
        let mut lock = self.meter.write().expect("poisoned lock");

        let gas = lock.gas_consumed_to_limit();
        lock.consume_gas(gas, DS::name().to_owned())
    }

    pub fn consume_gas(&mut self, amount: Gas) -> Result<(), GasErrors> {
        self.meter
            .write()
            .expect("poisoned lock")
            .consume_gas(amount, DS::name().to_owned())
    }

    pub fn is_out_of_gas(&self) -> bool {
        self.meter.read().expect("poisoned lock").is_out_of_gas()
    }
}

pub trait Descriptor: sealed::Sealed {
    fn name() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct BlockDescriptor;

impl Descriptor for BlockDescriptor {
    fn name() -> &'static str {
        "block gas meter"
    }
}

mod sealed {
    use super::BlockDescriptor;

    pub trait Sealed {}

    impl Sealed for BlockDescriptor {}
}
