pub mod check;
pub mod deliver;
pub mod re_check;

use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::tx::{raw::TxWithRaw, TxMessage},
};

use self::sealed::Sealed;

use super::TxContext;

pub trait ExecutionMode: Sealed {
    fn runnable<SK: StoreKey, DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<(), RunTxError>;

    fn run_ante_checks<
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;

    fn run_msg<
        'm,
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
    >(
        &mut self,
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
