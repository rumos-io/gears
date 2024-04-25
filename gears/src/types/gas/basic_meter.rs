use std::fmt::Display;

use super::{ErrorNegativeGasConsumed, Gas, GasErrors, GasRemaining, PlainGasMeter};

/// Basic gas meter.
#[derive(Debug, Clone)]
pub struct BasicGasMeter {
    limit: Gas,
    consumed: Gas,
}

impl BasicGasMeter {
    /// Create new `BasicGasMeter` with zero consumed gas.
    pub fn new(limit: Gas) -> Self {
        Self {
            limit,
            consumed: Gas(0),
        }
    }
}

impl PlainGasMeter for BasicGasMeter {
    fn gas_consumed(&self) -> Gas {
        self.consumed
    }

    fn gas_consumed_or_limit(&self) -> Gas {
        if self.is_past_limit() {
            self.limit
        } else {
            self.consumed
        }
    }

    fn gas_remaining(&self) -> GasRemaining {
        if self.is_past_limit() {
            GasRemaining::Some(Gas(0))
        } else {
            GasRemaining::Some(Gas(self.limit.0 - self.consumed.0))
        }
    }

    fn limit(&self) -> Option<Gas> {
        Some(self.limit)
    }

    fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasErrors> {
        if let Some(sum) = self.consumed.0.checked_add(amount.0) {
            if self.consumed > self.limit {
                Err(GasErrors::ErrorOutOfGas(descriptor.to_owned()))
            } else {
                self.consumed = Gas(sum);
                Ok(())
            }
        } else {
            self.consumed = Gas(u64::MAX);
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
        self.consumed > self.limit
    }

    fn is_out_of_gas(&self) -> bool {
        self.consumed >= self.limit
    }

    fn name(&self) -> &'static str {
        "gears basic gas meter"
    }
}

impl Display for BasicGasMeter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BasicGasMeter: limit {} consumed  {}",
            self.limit.0, self.consumed.0
        )
    }
}
