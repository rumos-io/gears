use store_crate::{
    database::{Database, PrefixDB},
    StoreKey, TransactionalMultiKVStore,
};
use tendermint::types::proto::event::Event;

use super::mode::{CheckTxMode, DeliverTxMode, ExecutionMode};
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::TransactionalContext,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

impl ExecutionMode for CheckTxMode {
    fn run_msg<
        'm,
        SK: StoreKey,
        DB: Database,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        ctx: &mut CTX,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().tx_caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks<
        SK: StoreKey,
        DB: Database,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        ctx: &mut CTX,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().tx_caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
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
        ctx: &mut CTX,
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
}
