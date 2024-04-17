use crate::types::{
    context::gas::{ConsumedToLimit, UnConsumed},
    gas::gas_meter::{Gas, GasErrors},
};

use super::TxContextWithGas;

// TODO: Gas refund?

impl<'a, DB, SK> TxContextWithGas<'a, DB, SK, UnConsumed> {
    pub fn gas_block_consume(
        self,
        amount: Gas,
    ) -> Result<Self, (GasErrors, TxContextWithGas<'a, DB, SK, ConsumedToLimit>)> {
        let Self {
            events,
            multi_store,
            height,
            header,
            block_gas_meter,
        } = self;

        let result = block_gas_meter.consume_gas(amount);

        match result {
            Ok(block_gas_meter) => Ok(Self {
                events,
                multi_store,
                height,
                header,
                block_gas_meter,
            }),
            Err((e, block_gas_meter)) => Err((
                e,
                TxContextWithGas {
                    events,
                    multi_store,
                    height,
                    header,
                    block_gas_meter,
                },
            )),
        }
    }

    pub fn gas_block_consume_to_limit(
        self,
    ) -> Result<TxContextWithGas<'a, DB, SK, ConsumedToLimit>, (GasErrors, Self)> {
        let Self {
            events,
            multi_store,
            height,
            header,
            block_gas_meter,
        } = self;

        let result = block_gas_meter.consume_to_limit();

        match result {
            Ok(block_gas_meter) => Ok(TxContextWithGas {
                events,
                multi_store,
                height,
                header,
                block_gas_meter,
            }),
            Err((e, block_gas_meter)) => Err((
                e,
                Self {
                    events,
                    multi_store,
                    height,
                    header,
                    block_gas_meter,
                },
            )),
        }
    }
}
