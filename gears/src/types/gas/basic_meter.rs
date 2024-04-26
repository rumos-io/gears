use std::fmt::Display;

use super::{ErrorNegativeGasConsumed, FiniteGas, Gas, GasErrors, PlainGasMeter};

/// Basic gas meter.
#[derive(Debug, Clone)]
pub struct BasicGasMeter {
    limit: FiniteGas,
    consumed: FiniteGas,
}

impl BasicGasMeter {
    /// Create new `BasicGasMeter` with zero consumed gas.
    pub fn new(limit: FiniteGas) -> Self {
        Self {
            limit,
            consumed: FiniteGas(0),
        }
    }
}

impl PlainGasMeter for BasicGasMeter {
    fn gas_consumed(&self) -> FiniteGas {
        self.consumed
    }

    fn gas_consumed_or_limit(&self) -> FiniteGas {
        if self.is_past_limit() {
            self.limit
        } else {
            self.consumed
        }
    }

    fn gas_remaining(&self) -> Gas {
        if self.is_past_limit() {
            Gas::Finite(FiniteGas(0))
        } else {
            Gas::Finite(FiniteGas(self.limit.0 - self.consumed.0))
        }
    }

    fn limit(&self) -> Gas {
        Gas::Finite(self.limit)
    }

    fn consume_gas(&mut self, amount: FiniteGas, descriptor: &str) -> Result<(), GasErrors> {
        if let Some(sum) = self.consumed.0.checked_add(amount.0) {
            if self.consumed > self.limit {
                Err(GasErrors::ErrorOutOfGas(descriptor.to_owned()))
            } else {
                self.consumed = FiniteGas(sum);
                Ok(())
            }
        } else {
            self.consumed = FiniteGas(u64::MAX);
            Err(GasErrors::ErrorGasOverflow(descriptor.to_owned()))
        }
    }

    fn refund_gas(
        &mut self,
        amount: FiniteGas,
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
