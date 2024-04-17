pub mod re_check;
pub mod check;
pub mod deliver;

use store_crate::{
    database::{Database, PrefixDB},
    StoreKey,
};
use tendermint::types::proto::event::Event;

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::TransactionalContext,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use self::sealed::Sealed;

pub trait ExecutionMode: Sealed {
    fn runnable(&self) -> Result<(), RunTxError>;

    fn run_msg<
        'm,
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        &mut self,
        ctx: &mut CTX,
        handler: &AH,
        msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError>;

    fn run_ante_checks<
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        &mut self,
        ctx: &mut CTX,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;
}


mod sealed {
    use super::{check::CheckTxMode, deliver::DeliverTxMode};

    pub trait Sealed {}

    impl Sealed for CheckTxMode {}
    // impl Sealed for ReCheckTxMode {}
    impl Sealed for DeliverTxMode {}
}
