use std::sync::Arc;

use store_crate::types::multi::commit::CommitMultiStore;
use store_crate::TransactionalMultiKVStore;
use store_crate::{database::Database, StoreKey};
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
pub struct DeliverTxMode<DB, SK> {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
    pub(crate) multi_store: CommitMultiStore<DB, SK>,
}

impl<DB: Database, SK: StoreKey> DeliverTxMode<DB, SK> {
    pub fn new(db: Arc<DB>, max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::<InfiniteGasMeter>::default(),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
            multi_store: CommitMultiStore::new(db),
        }
    }
}

impl<DB: Database + Sync + Send, SK: StoreKey> ExecutionMode<DB, SK> for DeliverTxMode<DB, SK> {
    fn build_ctx(&mut self, height: u64, header: Header, fee: Option<&Fee>) -> TxContext<'_, DB, SK> {
        TxContext::new(
            &mut self.multi_store,
            height,
            header,
            build_tx_gas_meter(height, fee),
            &mut self.block_gas_meter,
        )
    }

    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        for msg in msgs {
            handler
                .tx(ctx, msg)
                .inspect_err(|_| ctx.multi_store_mut().tx_caches_clear())
                .map_err(|e| RunTxError::Custom(e.to_string()))?;
        }

        let events = ctx.events_drain();
        ctx.multi_store_mut().tx_cache_to_block();

        Ok(events)
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        match handler.run_ante_checks(ctx, tx_with_raw) {
            Ok(_) => {
                ctx.multi_store_mut().tx_cache_to_block();
            }
            Err(e) => {
                ctx.multi_store_mut().tx_caches_clear();
                return Err(RunTxError::Custom(e.to_string()));
            }
        };

        Ok(())
    }

    fn runnable(ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }
}
