use std::marker::PhantomData;

use crate::types::context::ExecMode;

use crate::types::gas::gas_meter::{GasErrors, GasMeter};

#[derive(Debug, Clone)]
pub struct CtxGasMeter<GM, ST> {
    meter: GM,
    pub mode: ExecMode,
    _state: PhantomData<ST>,
}

impl<GM: GasMeter> CtxGasMeter<GM, UnConsumed> {
    pub fn new(meter: GM, mode: ExecMode) -> Self {
        Self {
            meter,
            mode,
            _state: PhantomData,
        }
    }
}

impl<GM: GasMeter> CtxGasMeter<GM, UnConsumed> {
    pub fn consume_to_limit(self) -> Result<CtxGasMeter<GM, ConsumedToLimit>, GasErrors> {
        let CtxGasMeter {
            mut meter,
            mode,
            _state: _,
        } = self;

        if mode == ExecMode::Deliver {
            let gas = meter.gas_consumed_to_limit();
            meter.consume_gas(gas, "block gas meter".to_owned())?;
        }

        Ok(CtxGasMeter {
            meter,
            mode,
            _state: PhantomData,
        })
    }
}

pub trait MeterState: sealed::Sealed {
    /// Currently is not possible to implement Drop only for specific state so this is hack for it
    fn is_consumed() -> bool;
}

pub struct ConsumedToLimit;

impl MeterState for ConsumedToLimit {
    fn is_consumed() -> bool {
        true
    }
}

pub struct UnConsumed;

impl MeterState for UnConsumed {
    fn is_consumed() -> bool {
        false
    }
}

// impl<GM: GasMeter, S: MeterState> Drop for CtxGasMeter<GM, S> {
//     fn drop(&mut self) {
//         if S::is_consumed() {
//         } else {
//             if self.mode == ExecMode::Deliver {
//                 let gas = self.meter.gas_consumed_to_limit();
//                 self.meter.consume_gas(gas, "block gas meter".to_owned());
//             }
//         }
//     }
// }

mod sealed {
    use super::{ConsumedToLimit, UnConsumed};

    pub trait Sealed {}

    impl Sealed for ConsumedToLimit {}

    impl Sealed for UnConsumed {}
}
