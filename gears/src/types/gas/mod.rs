/// Module for basic gas meter
pub mod basic_meter;
/// Module for config of gas meter.
pub mod config;
/// Module for infinite gas meter.
pub mod infinite_meter;
// Different descriptor for gas meter
pub mod descriptor;
// Kinds of gas meters
pub mod kind;

use std::fmt::Debug;
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::error::POISONED_LOCK;

use self::{descriptor::MeterDescriptor, kind::MeterKind};

#[no_link]
extern crate derive_more;

use derive_more::{Add, Deref, Display, From};

#[derive(
    Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, From, Add, Display, Deref,
)]
pub struct Gas(u64);

impl Gas {
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    pub const MAX_GAS: Gas = Gas::new(u64::MAX);
}

#[derive(Debug, thiserror::Error)]
pub enum GasErrors {
    #[error("Out of gas: {0}")]
    ErrorOutOfGas(String),
    #[error("Gas overflow: {0}")]
    ErrorGasOverflow(String),
}

#[derive(Debug)]
pub struct ErrorNegativeGasConsumed(pub String);

pub enum GasRemaining {
    NoLimit,
    None,
    Some(Gas),
}

pub trait PlainGasMeter: Send + Sync + Debug {
    /// Returns the amount of gas that was consumed by the gas meter instance.
    fn gas_consumed(&self) -> Gas;
    /// Returns the amount of gas that was consumed by gas meter instance, or the limit if it is reached.
    fn gas_consumed_or_limit(&self) -> Gas;
    /// Returns the gas left in the GasMeter. Returns `None` if gas meter is infinite.
    fn gas_remaining(&self) -> GasRemaining;
    /// Returns the limit of the gas meter instance. `None` if the gas meter is infinite.
    fn limit(&self) -> Option<Gas>;
    /// Consumes the amount of gas provided.
    /// If the gas overflows, it returns error with the descriptor message.
    /// If the gas meter is not infinite, it returns error  if gas consumed goes above the limit.
    fn consume_gas(&mut self, amount: Gas, descriptor: String) -> Result<(), GasErrors>;
    /// Deducts the given amount from the gas consumed.
    /// This functionality enables refunding gas to the transaction
    /// or block gas pools so that EVM-compatible chains can fully support the go-ethereum StateDB interface.
    fn refund_gas(
        &mut self,
        amount: Gas,
        descriptor: String,
    ) -> Result<(), ErrorNegativeGasConsumed>;
    /// Returns true if the amount of gas consumed by the gas meter instance is strictly above the limit, false otherwise.
    fn is_past_limit(&self) -> bool;
    /// Returns true if the amount of gas consumed by the gas meter instance is above or equal to the limit, false otherwise.
    fn is_out_of_gas(&self) -> bool;
}

/// Wrapper around any gas meter which prevents usage of gas over limit with type system
#[derive(Debug, Clone)]
pub struct GasMeter<DS> {
    meter: Arc<RwLock<Box<dyn PlainGasMeter>>>, // TODO: Smth other?
    _descriptor: PhantomData<DS>,
}

impl<DS> GasMeter<DS> {
    pub fn new(meter: Arc<RwLock<Box<dyn PlainGasMeter>>>) -> Self {
        Self {
            meter,
            _descriptor: PhantomData,
        }
    }
}

impl<DS: MeterKind> GasMeter<DS> {
    pub fn replace_meter(&mut self, meter: Box<dyn PlainGasMeter>) {
        self.meter.clear_poison(); // We replace gas meter so we shouldn't worry about poison
        let _ = std::mem::replace(&mut *self.meter.write().expect("Unreachable poison"), meter);
    }

    pub fn consume_to_limit<MD: MeterDescriptor>(&mut self) -> Result<(), GasErrors> {
        let mut lock = self.meter.write().expect(POISONED_LOCK);

        let gas = lock.gas_consumed_or_limit();
        lock.consume_gas(gas, MD::name().to_owned())
    }

    pub fn consume_gas<MD: MeterDescriptor>(&mut self, amount: Gas) -> Result<(), GasErrors> {
        self.meter
            .write()
            .expect(POISONED_LOCK)
            .consume_gas(amount, MD::name().to_owned())
    }

    pub fn is_out_of_gas(&self) -> bool {
        self.meter.read().expect(POISONED_LOCK).is_out_of_gas()
    }

    pub fn limit(&self) -> Option<Gas> {
        self.meter.read().expect(POISONED_LOCK).limit()
    }

    pub fn gas_remaining(&self) -> GasRemaining {
        self.meter.read().expect(POISONED_LOCK).gas_remaining()
    }
}
