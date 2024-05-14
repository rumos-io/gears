pub mod check;
pub mod deliver;
pub mod re_check;

use database::Database;
use store_crate::{types::multi::MultiBank, CacheKind, StoreKey};
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

pub trait ExecutionMode: Sealed {
    fn build_ctx<DB: Database, SK: StoreKey>(
        &mut self,
        store: MultiBank<DB, SK, CacheKind>,
        height: u64,
        header: Header,
        fee: Option<&Fee>,
    ) -> TxContext<'_, DB, SK>;

    fn runnable<DB: Database, SK: StoreKey>(
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<(), RunTxError>;

    fn run_ante_checks<
        DB: Database + Send + Sync,
        SK: StoreKey,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;

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
    ) -> Result<Vec<Event>, RunTxError>;
}

mod sealed {
    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl Sealed for CheckTxMode {}
    // impl Sealed for ReCheckTxMode {}
    impl Sealed for DeliverTxMode {}
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
