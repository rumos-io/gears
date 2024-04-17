use store_crate::{
    database::{Database, PrefixDB},
    StoreKey,
};
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::TransactionalContext,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use store_crate::TransactionalMultiKVStore;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CheckTxMode;

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
        &mut self,
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
        &mut self,
        ctx: &mut CTX,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().tx_caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(&self) -> Result<(), RunTxError> {
        Ok(())
    }
}
