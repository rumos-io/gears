use database::Database;
use kv_store::TransactionStore;
use kv_store::types::multi::MultiBank;
use tendermint::types::proto::event::Event;
use tendermint::types::proto::header::Header;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::errors::RunTxError,
    context::{TransactionalContext, tx::TxContext},
    types::tx::raw::TxWithRaw,
};
use crate::baseapp::ConsensusParams;
use crate::baseapp::options::NodeOptions;
use crate::types::auth::fee::Fee;
use crate::types::gas::{Gas, GasMeter};
use crate::types::gas::basic_meter::BasicGasMeter;
use crate::types::gas::infinite_meter::InfiniteGasMeter;
use crate::types::gas::kind::BlockKind;

use super::{build_tx_gas_meter, ExecutionMode};

#[derive(Debug)]
pub struct DeliverTxMode<DB, AH: ABCIHandler> {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
    pub(crate) multi_store: MultiBank<DB, AH::StoreKey, TransactionStore>,
}

impl<DB, AH: ABCIHandler> DeliverTxMode<DB, AH> {
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

impl<DB: Database + Sync + Send, AH: ABCIHandler> ExecutionMode<DB, AH> for DeliverTxMode<DB, AH> {
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
            false,
            options,
        )
    }

    fn run_msg<'m>(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        msgs: impl Iterator<Item=&'m AH::Message>,
    ) -> Result<Vec<Event>, RunTxError> {
        for msg in msgs {
            handler
                .tx(ctx, msg)
                .inspect_err(|_| ctx.multi_store_mut().clear_tx_cache()) // This may be ignored as `CacheKind` MS gets dropped at end of `run_tx`, but I want to be 100% sure
                .map_err(|e| RunTxError::Custom(e.to_string()))?;
        }

        let events = ctx.events_drain();

        Ok(events)
    }

    fn run_ante_checks(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<AH::Message>,
    ) -> Result<(), RunTxError> {
        match handler.run_ante_checks(ctx, tx_with_raw) {
            Ok(_) => Ok(()),
            Err(e) => {
                ctx.multi_store_mut().clear_tx_cache();
                Err(RunTxError::Custom(e.to_string()))
            }
        }
    }

    fn runnable(ctx: &mut TxContext<'_, DB, AH::StoreKey>) -> Result<(), RunTxError> {
        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }

    fn commit(mut ctx: TxContext<'_, DB, AH::StoreKey>) {
        ctx.multi_store_mut().upgrade_cache();
    }
}
