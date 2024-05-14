use database::Database;
use store_crate::{types::multi::MultiBank, CacheKind, StoreKey, TransactionalMultiKVStore};
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
pub struct CheckTxMode {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
}

impl CheckTxMode {
    pub fn new(max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
        }
    }
}

impl ExecutionMode for CheckTxMode {
    fn build_ctx<DB: Database, SK: StoreKey>(
        &mut self,
        store: MultiBank<DB, SK, CacheKind>,
        height: u64,
        header: Header,
        fee: Option<&Fee>,
    ) -> TxContext<'_, DB, SK> {
        TxContext::new(
            store,
            height,
            header,
            build_tx_gas_meter(height, fee),
            &mut self.block_gas_meter,
        )
    }

    fn run_msg<
        'm,
        DB: Database,
        SK: StoreKey,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        ctx: &mut TxContext<'_, DB, SK>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks<
        DB: Database,
        SK: StoreKey,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable<DB: Database, SK: StoreKey>(
        _: &mut TxContext<'_, DB, SK>,
    ) -> Result<(), RunTxError> {
        Ok(())
    }
}
