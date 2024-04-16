use std::fmt::Display;

use super::gas_meter::{ErrorNegativeGasConsumed, Gas, GasErrors, GasMeter};

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

impl GasMeter for BasicGasMeter {
    fn gas_consumed(&self) -> Gas {
        self.consumed
    }

    fn gas_consumed_to_limit(&self) -> Gas {
        if self.is_past_limit() {
            self.limit
        } else {
            self.consumed
        }
    }

    fn gas_remaining(&self) -> Gas {
        if self.is_past_limit() {
            Gas(0)
        } else {
            Gas(self.limit.0 - self.consumed.0)
        }
    }

    fn limit(&self) -> Gas {
        self.limit
    }

    fn consume_gas(&mut self, amount: Gas, descriptor: String) -> Result<(), GasErrors> {
        if let Some(sum) = self.consumed.0.checked_add(amount.0) {
            if self.consumed > self.limit {
                Err(GasErrors::ErrorOutOfGas(descriptor))
            } else {
                self.consumed = Gas(sum);
                Ok(())
            }
        } else {
            self.consumed = Gas(u64::MAX);
            Err(GasErrors::ErrorGasOverflow(descriptor))
        }
    }

    fn refund_gas(
        &mut self,
        amount: Gas,
        descriptor: String,
    ) -> Result<(), ErrorNegativeGasConsumed> {
        if self.consumed < amount {
            Err(ErrorNegativeGasConsumed(descriptor))
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
