pub mod check;
pub mod deliver;
pub mod re_check;

use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::tx::TxContext,
        header::Header,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use self::sealed::Sealed;

pub trait ExecutionMode<DB: Database, SK: StoreKey>: Sealed {
    fn runnable(&self, ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError>;

    fn build_ctx(&mut self, height: u64, header: Header) -> TxContext<'_, DB, SK>;

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;

    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError>;
}

mod sealed {
    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl<DB, SK> Sealed for CheckTxMode<DB, SK> {}
    // impl Sealed for ReCheckTxMode {}
    impl<DB, SK> Sealed for DeliverTxMode<DB, SK> {}
}
