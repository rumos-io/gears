pub mod check;
pub mod deliver;
pub mod re_check;

use kv_store::{types::multi::MultiBank, ApplicationStore};
use tendermint::types::proto::{event::Event, header::Header};

use self::sealed::Sealed;
use super::{options::NodeOptions, params::ConsensusParams};
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

pub trait ExecutionMode<DB, AH: ABCIHandler>: Sealed {
    fn build_ctx(
        &mut self,
        height: u64,
        header: Header,
        consensus_params: ConsensusParams,
        fee: Option<&Fee>,
        options: NodeOptions,
    ) -> TxContext<'_, DB, AH::StoreKey>;

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

    fn commit(
        ctx: TxContext<'_, DB, AH::StoreKey>,
        global_ms: &mut MultiBank<DB, AH::StoreKey, ApplicationStore>,
    );
}

mod sealed {
    use crate::application::handlers::node::ABCIHandler;

    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl<DB, AH: ABCIHandler> Sealed for CheckTxMode<DB, AH> {}
    // impl Sealed for ReCheckTxMode {}
    impl<DB, AH: ABCIHandler> Sealed for DeliverTxMode<DB, AH> {}
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
