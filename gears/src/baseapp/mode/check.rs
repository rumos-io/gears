use database::Database;
use store_crate::{types::multi::MultiBank, StoreKey, TransactionStore, TransactionalMultiKVStore};
use tendermint::types::proto::event::Event;

use super::{build_tx_gas_meter, ExecutionMode};
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        auth::fee::Fee,
        context::{tx::TxContext, TransactionalContext},
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::BlockKind, Gas,
            GasMeter,
        },
        header::Header,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

#[derive(Debug)]
pub struct CheckTxMode<DB, SK> {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
    pub(crate) multi_store: MultiBank<DB, SK, TransactionStore>,
}

impl<DB, SK> CheckTxMode<DB, SK> {
    pub fn new(max_gas: Gas, multi_store: MultiBank<DB, SK, TransactionStore>) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
            multi_store,
        }
    }
}

impl<DB: Database, SK: StoreKey> ExecutionMode<DB, SK> for CheckTxMode<DB, SK> {
    fn build_ctx(
        &mut self,
        height: u64,
        header: Header,
        fee: Option<&Fee>,
    ) -> TxContext<'_, DB, SK> {
        TxContext::new(
            &mut self.multi_store,
            height,
            header,
            build_tx_gas_meter(height, fee),
            &mut self.block_gas_meter,
            true,
        )
    }

    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(_: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        Ok(())
    }

    fn commit(
        _ctx: TxContext<'_, DB, SK>,
        _global_ms: &mut MultiBank<DB, SK, store_crate::ApplicationStore>,
    ) {
    }
}
