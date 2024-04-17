use crate::types::{
    context::gas::{ConsumedToLimit, UnConsumed},
    gas::gas_meter::{Gas, GasErrors, GasMeter},
};

use super::TxContext2;

// TODO: Gas refund?

impl<'a, DB, SK, GM: GasMeter> TxContext2<'a, DB, SK, GM, UnConsumed> {
    pub fn gas_block_consume(
        self,
        amount: Gas,
    ) -> Result<Self, (GasErrors, TxContext2<'a, DB, SK, GM, ConsumedToLimit>)> {
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
                TxContext2 {
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
    ) -> Result<TxContext2<'a, DB, SK, GM, ConsumedToLimit>, (GasErrors, Self)> {
        let Self {
            events,
            multi_store,
            height,
            header,
            block_gas_meter,
        } = self;

        let result = block_gas_meter.consume_to_limit();

        match result {
            Ok(block_gas_meter) => Ok(TxContext2 {
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
