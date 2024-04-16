use std::ops::Deref;

use crate::types::context::ExecMode;

use crate::types::gas::gas_meter::{GasErrors, GasMeter};

#[derive(Debug, Clone)]
pub struct CtxGasMeter<GM> {
    meter: GM,
    pub mode: ExecMode,
}

impl<GM: GasMeter> CtxGasMeter<GM> {
    pub fn new(meter: GM, mode: ExecMode) -> Self {
        Self { meter, mode }
    }

    pub fn consume_to_limit(&mut self) -> Result<(), GasErrors> {
        if self.mode == ExecMode::Deliver {
            let gas = self.meter.gas_consumed_to_limit();
            self.meter.consume_gas(gas, "block gas meter".to_owned())?;
        }

        Ok(())
    }
}

impl<GM> Deref for CtxGasMeter<GM> {
    type Target = GM;

    fn deref(&self) -> &Self::Target {
        &self.meter
    }
}
