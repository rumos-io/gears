use std::sync::{Arc, RwLock};

use store_crate::types::multi::MultiStore;
use store_crate::TransactionalMultiKVStore;
use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use crate::types::auth::fee::Fee;
use crate::types::context::tx::TxContext;
use crate::types::gas::basic_meter::BasicGasMeter;
use crate::types::gas::infinite_meter::InfiniteGasMeter;
use crate::types::gas::kind::BlockMeterKind;
use crate::types::gas::{Gas, GasMeter, PlainGasMeter};
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

    pub(crate) fn build_tx_gas_meter(fee: &Fee, block_height: u64) -> Box<dyn PlainGasMeter> {
        if block_height == 0 {
            Box::new(InfiniteGasMeter::default())
        } else {
            Box::new(BasicGasMeter::new(Gas::new(fee.gas_limit)))
        }
    }
}

impl<DB: Database + Sync + Send, SK: StoreKey> ExecutionMode<DB, SK> for DeliverTxMode<DB, SK> {
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
        ctx.multi_store_mut().tx_caches_write_then_clear();

        Ok(events)
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        match handler.run_ante_checks(ctx, tx_with_raw) {
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

    fn runnable(ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        if ctx.block_gas_meter.is_out_of_gas() {
            Err(RunTxError::OutOfGas)
        } else {
            Ok(())
        }
    }

    fn build_ctx<M: TxMessage>(
        &mut self,
        height: u64,
        header: &Header,
        tx: &TxWithRaw<M>,
    ) -> TxContext<'_, DB, SK> {
        let mut ctx = TxContext::new(
            &mut self.multi_store,
            height,
            header.clone(),
            self.block_gas_meter.clone(),
        );
        ctx.gas_meter
            .replace_meter(Self::build_tx_gas_meter(&tx.tx.auth_info.fee, height));

        ctx
    }
}
