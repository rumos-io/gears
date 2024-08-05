use tendermint::types::proto::event::Event;

use self::sealed::Sealed;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::errors::RunTxError,
    context::tx::TxContext,
    types::{
        auth::fee::Fee,
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::TxKind, GasMeter,
        },
        tx::raw::TxWithRaw,
    },
};

pub mod check;
pub mod deliver;

pub trait ExecutionMode<DB, AH: ABCIHandler>: Sealed {
    fn runnable(ctx: &mut TxContext<'_, DB, AH::StoreKey>) -> Result<(), RunTxError>;

    fn run_ante_checks(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<AH::Message>,
    ) -> Result<(), RunTxError>;

    fn run_msg<'m>(
        ctx: &mut TxContext<'_, DB, AH::StoreKey>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m AH::Message>,
    ) -> Result<Vec<Event>, RunTxError>;

    fn commit(ctx: TxContext<'_, DB, AH::StoreKey>);
}

mod sealed {
    use crate::application::handlers::node::ABCIHandler;

    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl<DB, AH: ABCIHandler> Sealed for CheckTxMode<DB, AH> {}
    impl<DB, AH: ABCIHandler> Sealed for DeliverTxMode<DB, AH> {}
}

pub(crate) fn build_tx_gas_meter(block_height: u32, fee: Option<&Fee>) -> GasMeter<TxKind> {
    if block_height == 0 {
        GasMeter::new(Box::<InfiniteGasMeter>::default())
    } else {
        GasMeter::new(Box::new(BasicGasMeter::new(
            fee.map(|e| e.gas_limit).unwrap_or_default(),
        )))
    }
}
