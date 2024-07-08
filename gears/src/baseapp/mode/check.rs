use database::Database;
use kv_store::{TransactionStore, types::multi::MultiBank};
use tendermint::types::proto::{event::Event, header::Header};

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, options::NodeOptions, params::ConsensusParams},
    context::{TransactionalContext, tx::TxContext},
    types::{
        auth::fee::Fee,
        gas::{
            basic_meter::BasicGasMeter, Gas, GasMeter, infinite_meter::InfiniteGasMeter,
            kind::BlockKind,
        },
        tx::raw::TxWithRaw,
    },
};

use super::{build_tx_gas_meter, ExecutionMode};

#[derive(Debug)]
pub struct CheckTxMode<DB, AH: ABCIHandler> {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
    pub(crate) multi_store: MultiBank<DB, AH::StoreKey, TransactionStore>,
}

impl<DB, AH: ABCIHandler> CheckTxMode<DB, AH> {
    pub fn new(max_gas: Gas, multi_store: MultiBank<DB, AH::StoreKey, TransactionStore>) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
            multi_store,
        }
    }
}

impl<DB: Database, AH: ABCIHandler> ExecutionMode<DB, AH> for CheckTxMode<DB, AH> {
    fn build_ctx(
        &mut self,
        height: u32,
        header: Header,
        consensus_params: ConsensusParams,
        fee: Option<&Fee>,
        options: NodeOptions,
    ) -> TxContext<'_, DB, AH::StoreKey> {
        TxContext::new(
            &mut self.multi_store,
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
        _msgs: impl Iterator<Item=&'m AH::Message>,
    ) -> Result<Vec<Event>, RunTxError> {
        Ok(ctx.events_drain())
    }

    fn run_ante_checks(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<AH::Message>,
    ) -> Result<(), RunTxError> {
        handler
            .run_ante_checks(ctx, tx_with_raw)
            .inspect_err(|_| ctx.multi_store_mut().clear_tx_cache())
            .map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(_: &mut TxContext<'_, DB, AH::StoreKey>) -> Result<(), RunTxError> {
        Ok(())
    }

    fn commit(mut ctx: TxContext<'_, DB, AH::StoreKey>) {
        ctx.multi_store_mut().upgrade_cache();
    }
}
