pub mod check;
pub mod deliver;
pub mod re_check;

use database::Database;
use store_crate::StoreKey;
use tendermint::types::proto::event::Event;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        auth::fee::Fee,
        context::tx::TxContext,
        gas::{
            basic_meter::BasicGasMeter,
            infinite_meter::InfiniteGasMeter,
            kind::{BlockKind, TxKind},
            GasMeter,
        },
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use self::sealed::Sealed;

pub trait ExecutionMode<DB: Database, SK: StoreKey>: Sealed {
    fn block_gas_meter_mut(&mut self) -> &mut GasMeter<BlockKind>;

    fn build_tx_gas_meter(fee: &Fee, block_height: u64) -> GasMeter<TxKind> {
        if block_height == 0 {
            GasMeter::new(Box::<InfiniteGasMeter>::default())
        } else {
            GasMeter::new(Box::new(BasicGasMeter::new(fee.gas_limit)))
        }
    }

    fn runnable(&self, ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError>;

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;

    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError>;
}

mod sealed {
    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl Sealed for CheckTxMode {}
    // impl Sealed for ReCheckTxMode {}
    impl Sealed for DeliverTxMode {}
}
