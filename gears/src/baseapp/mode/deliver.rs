use database::Database;
use kv_store::bank::multi::TransactionMultiBank;
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::types::gas::basic_meter::BasicGasMeter;
use crate::types::gas::infinite_meter::InfiniteGasMeter;
use crate::types::gas::kind::BlockKind;
use crate::types::gas::{Gas, GasMeter};
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::errors::RunTxError,
    context::{tx::TxContext, TransactionalContext},
    types::tx::raw::TxWithRaw,
};

#[derive(Debug)]
pub struct DeliverTxMode<DB, AH: ABCIHandler> {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
    pub(crate) multi_store: TransactionMultiBank<DB, AH::StoreKey>,
}

impl<DB, AH: ABCIHandler> DeliverTxMode<DB, AH> {
    pub fn new(max_gas: Gas, multi_store: TransactionMultiBank<DB, AH::StoreKey>) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
            multi_store,
        }
    }
}

impl<DB: Database, AH: ABCIHandler> ExecutionMode<DB, AH> for DeliverTxMode<DB, AH> {
    fn run_msg<'m>(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m AH::Message>,
    ) -> Result<Vec<Event>, RunTxError> {
        for msg in msgs {
            handler
                .msg(ctx, msg)
                .inspect_err(|_| ctx.multi_store_mut().clear_cache())?
        }

        Ok(ctx.events_drain())
    }

    fn run_ante_checks(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<AH::Message>,
    ) -> Result<(), RunTxError> {
        let result = handler
            .run_ante_checks(ctx, tx_with_raw, false)
            .map_err(RunTxError::from);

        ctx.multi_store_mut().upgrade_cache();

        result
    }

    fn runnable(ctx: &mut TxContext<'_, DB, AH::StoreKey>) -> Result<(), RunTxError> {
        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfBlockGas)
        } else {
            Ok(())
        }
    }
}
