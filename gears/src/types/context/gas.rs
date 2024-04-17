use std::{marker::PhantomData, ops::Deref};

use crate::types::gas::gas_meter::{Gas, GasErrors, GasMeter};

/// Wrapper around any gas meter which prevents usage of gas over limit with type system
#[derive(Debug, Clone)]
pub struct CtxGasMeter<GM, ST, DS> {
    meter: GM,
    _state: std::marker::PhantomData<ST>,
    _descriptor: std::marker::PhantomData<DS>,
}

impl<GM, ST, DS> CtxGasMeter<GM, ST, DS> {
    pub fn new(meter: GM) -> Self {
        Self {
            meter,
            _state: PhantomData,
            _descriptor: PhantomData,
        }
    }
}

impl<GM, ST, DS> Deref for CtxGasMeter<GM, ST, DS> {
    type Target = GM;

    fn deref(&self) -> &Self::Target {
        &self.meter
    }
}

impl<GM: GasMeter, DS: Descriptor> CtxGasMeter<GM, UnConsumed, DS> {
    pub fn consume_to_limit(
        mut self,
    ) -> Result<CtxGasMeter<GM, ConsumedToLimit, DS>, (GasErrors, Self)> {
        let gas = self.meter.gas_consumed_to_limit();
        let result = self.meter.consume_gas(gas, DS::name().to_owned());

        match result {
            Ok(_) => Ok(CtxGasMeter {
                meter: self.meter,
                _state: std::marker::PhantomData,
                _descriptor: std::marker::PhantomData,
            }),
            Err(e) => Err((e, self)),
        }
    }

    pub fn consume_gas(
        mut self,
        amount: Gas,
    ) -> Result<Self, (GasErrors, CtxGasMeter<GM, ConsumedToLimit, DS>)> {
        let result = self.meter.consume_gas(amount, DS::name().to_owned());

        match result {
            Ok(_) => Ok(self),
            Err(e) => Err((
                e,
                CtxGasMeter {
                    meter: self.meter,
                    _state: std::marker::PhantomData,
                    _descriptor: std::marker::PhantomData,
                },
            )),
        }
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

mod sealed {
    use super::{BlockDescriptor, ConsumedToLimit, UnConsumed};

    pub trait Sealed {}

    impl Sealed for ConsumedToLimit {}

    impl Sealed for UnConsumed {}

    impl Sealed for BlockDescriptor {}
}
