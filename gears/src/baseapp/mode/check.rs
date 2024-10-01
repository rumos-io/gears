use database::Database;
use kv_store::bank::multi::TransactionMultiBank;
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::errors::RunTxError,
    context::{tx::TxContext, TransactionalContext},
    types::{
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
    pub(crate) multi_store: TransactionMultiBank<DB, AH::StoreKey>,
}

impl<DB, AH: ABCIHandler> CheckTxMode<DB, AH> {
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

impl<DB: Database, AH: ABCIHandler> ExecutionMode<DB, AH> for CheckTxMode<DB, AH> {
    fn run_msg<'m>(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m AH::Message>,
    ) -> Result<Vec<Event>, RunTxError> {
        Ok(ctx.events_drain())
    }

    fn run_ante_checks(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<AH::Message>,
    ) -> Result<(), RunTxError> {
        let result = handler
            .run_ante_checks(ctx, tx_with_raw, true)
            .map_err(RunTxError::from);

        ctx.multi_store_mut().upgrade_cache();

        result
    }

    fn runnable(_: &mut TxContext<'_, DB, AH::StoreKey>) -> Result<(), RunTxError> {
        Ok(())
    }
}
