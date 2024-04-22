use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::{tx::TxContext, TransactionalContext},
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
    >(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
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
    >(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().tx_caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable<SK: StoreKey, DB: Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<(), RunTxError> {
        Ok(())
    }
}
