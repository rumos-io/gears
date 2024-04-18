/// Module for basic gas meter
pub mod basic_meter;
/// Module for config of gas meter.
pub mod config;
/// Module for infinite gas meter.
pub mod infinite_meter;

use std::fmt::Debug;

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

pub trait GasMeter: Send + Sync + Debug {
    /// Returns the amount of gas that was consumed by the gas meter instance.
    fn gas_consumed(&self) -> Gas;
    /// Returns the amount of gas that was consumed by gas meter instance, or the limit if it is reached.
    fn gas_consumed_to_limit(&self) -> Gas;
    /// Returns the gas left in the GasMeter.
    fn gas_remaining(&self) -> Gas;
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
