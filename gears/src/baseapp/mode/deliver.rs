use std::sync::{Arc, RwLock};

use store_crate::types::multi::MultiStore;
use store_crate::TransactionalMultiKVStore;
use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use crate::types::context::tx::TxContext;
use crate::types::gas::basic_meter::BasicGasMeter;
use crate::types::gas::infinite_meter::InfiniteGasMeter;
use crate::types::gas::kind::BlockMeterKind;
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

use super::ExecutionMode;

#[derive(Debug)]
pub struct DeliverTxMode<DB, SK> {
    pub(crate) block_gas_meter: GasMeter<BlockMeterKind>,
    pub(crate) multi_store: MultiStore<DB, SK>,
}

impl<DB: Database, SK: StoreKey> DeliverTxMode<DB, SK> {
    pub fn new(multi_store: MultiStore<DB, SK>, max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas > Gas::new(0) {
                true => Arc::new(RwLock::new(Box::new(InfiniteGasMeter::default()))),
                false => Arc::new(RwLock::new(Box::new(BasicGasMeter::new(max_gas)))),
            }),
            multi_store,
        }
    }
}

impl<DB: Database + Sync + Send, SK: StoreKey> ExecutionMode<DB, SK> for DeliverTxMode<DB, SK> {
    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        &mut self,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
        height: u64,
        header: &Header,
    ) -> Result<Vec<Event>, RunTxError> {
        let mut ctx = self.build_ctx(height, header);

        for msg in msgs {
            handler
                .tx(&mut ctx, msg)
                .inspect_err(|_| ctx.multi_store_mut().tx_caches_clear())
                .map_err(|e| RunTxError::Custom(e.to_string()))?;
        }

        let events = ctx.events_drain();
        ctx.multi_store_mut().tx_caches_write_then_clear();

        Ok(events)
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        &mut self,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
        height: u64,
        header: &Header,
    ) -> Result<(), RunTxError> {
        let mut ctx = self.build_ctx(height, header);

        match handler.run_ante_checks(&mut ctx, tx_with_raw) {
            Ok(_) => {
                ctx.multi_store_mut().tx_caches_write_then_clear();
            }
            Err(e) => {
                ctx.multi_store_mut().tx_caches_clear();
                return Err(RunTxError::Custom(e.to_string()));
            }
        };

        Ok(())
    }

    fn runnable(&mut self, height: u64, header: &Header) -> Result<(), RunTxError> {
        let ctx = self.build_ctx(height, header);

        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }

    fn build_ctx(&mut self, height: u64, header: &Header) -> TxContext<'_, DB, SK> {
        TxContext::new(
            &mut self.multi_store,
            height,
            header.clone(),
            self.block_gas_meter.clone(),
        )
    }
}
