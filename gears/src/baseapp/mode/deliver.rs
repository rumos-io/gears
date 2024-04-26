use store_crate::TransactionalMultiKVStore;
use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use crate::types::context::tx::TxContext;
use crate::types::gas::basic_meter::BasicGasMeter;
use crate::types::gas::infinite_meter::InfiniteGasMeter;
use crate::types::gas::kind::BlockKind;
use crate::types::gas::{Gas, GasMeter};
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::TransactionalContext,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use super::ExecutionMode;

#[derive(Debug)]
pub struct DeliverTxMode {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
}

impl DeliverTxMode {
    pub fn new(max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas > Gas::new(0) {
                true => Box::new(InfiniteGasMeter::default()),
                false => Box::new(BasicGasMeter::new(max_gas)),
            }),
        }
    }
}

impl<DB: Database + Sync + Send, SK: StoreKey> ExecutionMode<DB, SK> for DeliverTxMode {
    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        for msg in msgs {
            handler
                .tx(ctx, msg)
                .inspect_err(|_| ctx.multi_store_mut().tx_caches_clear())
                .map_err(|e| RunTxError::Custom(e.to_string()))?;
        }

        let events = ctx.events_drain();
        ctx.multi_store_mut().tx_caches_write_then_clear();

        Ok(events)
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        match handler.run_ante_checks(ctx, tx_with_raw) {
            Ok(_) => {
                ctx.multi_store_mut().tx_caches_write_then_clear();
            }
            Err(e) => {
                ctx.multi_store_mut().tx_caches_clear();
                return Err(RunTxError::Custom(e.to_string()));
            }
        };

        Ok(())
    }

    fn runnable(&self, _ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        if self.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }

    fn block_gas_meter_mut(&mut self) -> &mut GasMeter<BlockKind> {
        &mut self.block_gas_meter
    }
}
