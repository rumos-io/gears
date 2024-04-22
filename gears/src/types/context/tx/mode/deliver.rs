use store_crate::TransactionalMultiKVStore;
use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use crate::types::context::tx::TxContext;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::TransactionalContext,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use super::ExecutionMode;

#[derive(Debug, Clone)]
pub struct DeliverTxMode;

impl ExecutionMode for DeliverTxMode {
    fn run_msg<
        'm,
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        &mut self,
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

    fn run_ante_checks<
        SK: StoreKey,
        DB: Database,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
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

    fn runnable<SK: StoreKey, DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<(), RunTxError> {
        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }
}
