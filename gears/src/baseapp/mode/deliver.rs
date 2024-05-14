use database::Database;
use store_crate::types::multi::MultiBank;
use store_crate::{CacheKind, StoreKey, TransactionalMultiKVStore};
use tendermint::types::proto::event::Event;

use crate::types::auth::fee::Fee;
use crate::types::context::tx::TxContext;
use crate::types::gas::basic_meter::BasicGasMeter;
use crate::types::gas::infinite_meter::InfiniteGasMeter;
use crate::types::gas::kind::BlockKind;
use crate::types::gas::{Gas, GasMeter};
use crate::types::header::Header;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::TransactionalContext,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use super::{build_tx_gas_meter, ExecutionMode};

#[derive(Debug)]
pub struct DeliverTxMode {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
}

impl DeliverTxMode {
    pub fn new(max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
        }
    }
}

impl ExecutionMode for DeliverTxMode {
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
        DB: Database + Send + Sync,
        SK: StoreKey,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        for msg in msgs {
            handler
                .tx(ctx, msg)
                .inspect_err(|_| ctx.multi_store_mut().caches_clear()) // This may be ignored as `CacheKind` MS gets dropped at end of `run_tx`, but I want to be 100% sure
                .map_err(|e| RunTxError::Custom(e.to_string()))?;
        }

        let events = ctx.events_drain();

        Ok(events)
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
        match handler.run_ante_checks(ctx, tx_with_raw) {
            Ok(_) => Ok(()),
            Err(e) => {
                ctx.multi_store_mut().caches_clear();
                Err(RunTxError::Custom(e.to_string()))
            }
        }
    }

    fn runnable<DB: Database, SK: StoreKey>(
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<(), RunTxError> {
        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }
}
