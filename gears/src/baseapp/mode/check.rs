use std::sync::{Arc, RwLock};

use store_crate::{database::Database, types::multi::MultiStore, StoreKey};
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        auth::fee::Fee,
        context::{tx::TxContext, TransactionalContext},
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::BlockMeterKind,
            Gas, GasMeter, PlainGasMeter,
        },
        header::Header,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use store_crate::TransactionalMultiKVStore;

#[derive(Debug)]
pub struct CheckTxMode<DB, SK> {
    pub(crate) block_gas_meter: GasMeter<BlockMeterKind>,
    pub(crate) multi_store: MultiStore<DB, SK>,
}

impl<DB: Database, SK: StoreKey> CheckTxMode<DB, SK> {
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

impl<DB: Database, SK: StoreKey> ExecutionMode<DB, SK> for CheckTxMode<DB, SK> {
    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().tx_caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().tx_caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(_: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        Ok(())
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
