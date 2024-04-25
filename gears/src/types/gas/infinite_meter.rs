use std::fmt::Display;

use super::{ErrorNegativeGasConsumed, Gas, GasErrors, GasRemaining, PlainGasMeter};

/// Gas meter without consumption limit
#[derive(Debug, Clone)]
pub struct InfiniteGasMeter {
    consumed: Gas,
}

impl Default for InfiniteGasMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl InfiniteGasMeter {
    /// Create new `InfiniteGasMeter` with zero consumed gas.
    pub fn new() -> Self {
        Self { consumed: Gas(0) }
    }
}

impl PlainGasMeter for InfiniteGasMeter {
    fn gas_consumed(&self) -> Gas {
        self.consumed
    }

    fn gas_consumed_or_limit(&self) -> Gas {
        self.consumed
    }

    fn gas_remaining(&self) -> GasRemaining {
        GasRemaining::NoLimit
    }

    fn limit(&self) -> Option<Gas> {
        None
    }

    fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasErrors> {
        if let Some(sum) = self.consumed.0.checked_add(amount.0) {
            self.consumed = Gas(sum);
            Ok(())
        } else {
            Err(GasErrors::ErrorGasOverflow(descriptor.to_owned()))
        }
    }

    fn refund_gas(
        &mut self,
        amount: Gas,
        descriptor: &str,
    ) -> Result<(), ErrorNegativeGasConsumed> {
        if self.consumed < amount {
            Err(ErrorNegativeGasConsumed(descriptor.to_owned()))
        } else {
            self.consumed.0 -= amount.0;

            Ok(())
        }
    }

    fn is_past_limit(&self) -> bool {
        false
    }

    fn is_out_of_gas(&self) -> bool {
        false
    }
    
    fn name(&self) -> &'static str {
        "gears infinite meter"
    }
}

impl Display for InfiniteGasMeter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InfiniteGasMeter: consumed {}", self.consumed.0)
    }
}
