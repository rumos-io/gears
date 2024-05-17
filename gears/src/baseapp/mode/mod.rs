pub mod check;
pub mod deliver;
pub mod re_check;

use store_crate::{types::multi::MultiBank, CommitKind, StoreKey};
use tendermint::types::proto::event::Event;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        auth::fee::Fee,
        context::tx::TxContext,
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::TxKind, GasMeter,
        },
        header::Header,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use self::sealed::Sealed;

pub trait ExecutionMode<DB, SK: StoreKey>: Sealed {
    fn build_ctx(
        &mut self,
        height: u64,
        header: Header,
        fee: Option<&Fee>,
    ) -> TxContext<'_, DB, SK>;

    fn runnable(ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError>;

    fn run_ante_checks<
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G, QReq, QRes>,
        QReq,
        QRes,
    >(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;

    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G, QReq, QRes>, QReq, QRes>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError>;

    fn commit(ctx: TxContext<'_, DB, SK>, global_ms: &mut MultiBank<DB, SK, CommitKind>);
}

mod sealed {
    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl<DB, SK> Sealed for CheckTxMode<DB, SK> {}
    // impl Sealed for ReCheckTxMode {}
    impl<DB, SK> Sealed for DeliverTxMode<DB, SK> {}
}

fn build_tx_gas_meter(block_height: u64, fee: Option<&Fee>) -> GasMeter<TxKind> {
    if block_height == 0 {
        GasMeter::new(Box::<InfiniteGasMeter>::default())
    } else {
        GasMeter::new(Box::new(BasicGasMeter::new(
            fee.map(|e| e.gas_limit).unwrap_or_default(),
        )))
    }
}
