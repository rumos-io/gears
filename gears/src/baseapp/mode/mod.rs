use database::Database;
use tendermint::types::proto::event::Event;
use tendermint::types::proto::header::Header;

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

use super::{options::NodeOptions, ConsensusParams};

use self::sealed::Sealed;

pub mod check;
pub mod deliver;
pub mod re_check;

pub trait ExecutionMode<DB: Database, AH: ABCIHandler>: Sealed<DB, AH> {
    fn build_ctx(
        &mut self,
        height: u32,
        header: Header,
        consensus_params: ConsensusParams,
        fee: Option<&Fee>,
        options: NodeOptions,
    ) -> TxContext<'_, DB, AH::StoreKey> {
        let (ms, gas) = self.inner_fields();

        TxContext::new(
            ms,
            height,
            header,
            consensus_params,
            build_tx_gas_meter(height, fee),
            gas,
            true,
            options,
        )
    }

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

    fn commit(mut ctx: TxContext<'_, DB, AH::StoreKey>) {
        ctx.multi_store_mut().upgrade_cache();
    }
}

mod sealed {
    use database::Database;
    use kv::bank::multi::TransactionMultiBank;

    use crate::{
        application::handlers::node::ABCIHandler,
        types::gas::{kind::BlockKind, GasMeter},
    };

    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed<DB: Database, AH: ABCIHandler> {
        fn inner_fields(
            &mut self,
        ) -> (
            &mut TransactionMultiBank<DB, AH::StoreKey>,
            &mut GasMeter<BlockKind>,
        );
    }

    impl<DB: Database, AH: ABCIHandler> Sealed<DB, AH> for CheckTxMode<DB, AH> {
        fn inner_fields(
            &mut self,
        ) -> (
            &mut TransactionMultiBank<DB, <AH as ABCIHandler>::StoreKey>,
            &mut GasMeter<BlockKind>,
        ) {
            (&mut self.multi_store, &mut self.block_gas_meter)
        }
    }
    // impl Sealed for ReCheckTxMode {}
    impl<DB: Database, AH: ABCIHandler> Sealed<DB, AH> for DeliverTxMode<DB, AH> {
        fn inner_fields(
            &mut self,
        ) -> (
            &mut TransactionMultiBank<DB, <AH as ABCIHandler>::StoreKey>,
            &mut GasMeter<BlockKind>,
        ) {
            (&mut self.multi_store, &mut self.block_gas_meter)
        }
    }
}

fn build_tx_gas_meter(block_height: u32, fee: Option<&Fee>) -> GasMeter<TxKind> {
    if block_height == 0 {
        GasMeter::new(Box::<InfiniteGasMeter>::default())
    } else {
        GasMeter::new(Box::new(BasicGasMeter::new(
            fee.map(|e| e.gas_limit).unwrap_or_default(),
        )))
    }
}
