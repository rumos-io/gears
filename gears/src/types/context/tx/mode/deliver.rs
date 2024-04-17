use store_crate::TransactionalMultiKVStore;
use store_crate::{
    database::{Database, PrefixDB},
    StoreKey,
};
use tendermint::types::proto::event::Event;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::{
            gas::{BlockDescriptor, CtxGasMeter},
            TransactionalContext,
        },
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use super::ExecutionMode;

#[derive(Debug, Clone)]
pub struct DeliverTxMode {
    block_gas_meter: CtxGasMeter<BlockDescriptor>,
}

impl DeliverTxMode {
    pub fn new(block_gas_meter: CtxGasMeter<BlockDescriptor>) -> Self {
        Self { block_gas_meter }
    }
}

impl ExecutionMode for DeliverTxMode {
    fn run_msg<
        'm,
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        &mut self,
        ctx: &mut CTX,
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

    fn run_ante_checks<
        SK: StoreKey,
        DB: Database,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        &mut self,
        ctx: &mut CTX,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> // TODO: Return gasWanted
    {
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

    fn runnable(&self) -> Result<(), RunTxError> {
        if self.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }
}
