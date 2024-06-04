use database::Database;
use kv_store::{types::multi::MultiBank, ApplicationStore, TransactionStore};
use tendermint::types::proto::{event::Event, header::Header};

use super::{build_tx_gas_meter, ExecutionMode};
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, options::NodeOptions, params::ConsensusParams},
    context::{tx::TxContext, TransactionalContext},
    types::{
        auth::fee::Fee,
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::BlockKind, Gas,
            GasMeter,
        },
        tx::raw::TxWithRaw,
    },
};

#[derive(Debug)]
pub struct CheckTxMode<DB, AH: ABCIHandler> {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
    pub(crate) unpersisted_multi_store: MultiBank<DB, AH::StoreKey, TransactionStore>,
    pub(crate) persisted_multi_store: MultiBank<DB, AH::StoreKey, TransactionStore>,
}

impl<DB, AH: ABCIHandler> CheckTxMode<DB, AH> {
    pub fn new(max_gas: Gas, multi_store: &MultiBank<DB, AH::StoreKey, ApplicationStore>) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
            unpersisted_multi_store: multi_store.to_cache_kind(),
            persisted_multi_store: multi_store.to_cache_kind(),
        }
    }
}

impl<DB: Database, AH: ABCIHandler> ExecutionMode<DB, AH> for CheckTxMode<DB, AH> {
    fn multi_store(
        &mut self,
    ) -> &mut MultiBank<DB, <AH as ABCIHandler>::StoreKey, TransactionStore> {
        &mut self.unpersisted_multi_store
    }

    fn build_ctx(
        &mut self,
        height: u64,
        header: Header,
        consensus_params: ConsensusParams,
        fee: Option<&Fee>,
        options: NodeOptions,
    ) -> TxContext<'_, DB, AH::StoreKey> {
        TxContext::new(
            &mut self.unpersisted_multi_store,
            height,
            header,
            consensus_params,
            build_tx_gas_meter(height, fee),
            &mut self.block_gas_meter,
            true,
            options,
        )
    }

    fn run_msg<'m>(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m AH::Message>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<AH::Message>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(_: &mut TxContext<'_, DB, AH::StoreKey>) -> Result<(), RunTxError> {
        Ok(())
    }

    fn commit(&mut self, _global_ms: &mut MultiBank<DB, AH::StoreKey, kv_store::ApplicationStore>) {
        /*
            This is needed for case when `check_tx` fails and need to clear all cache,
            but it would delete cache not only for one call but for other calls leading
            to inconsistent state. So for this we need 2 stores in `CheckTxMode`
        */
        self.persisted_multi_store
            .caches_update(self.unpersisted_multi_store.caches_copy());
    }
}
