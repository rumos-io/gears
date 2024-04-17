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
    fn run_msg<
        'm,
        SK: StoreKey,
        DB: Database + Send + Sync,
        M: TxMessage,
        G: Genesis,
        AH: ABCIHandler<M, SK, G>,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
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
        ctx: &mut CTX,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CheckTxMode;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ReCheckTxMode;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DeliverTxMode;

mod sealed {
    use super::{CheckTxMode, DeliverTxMode, ReCheckTxMode};

    pub trait Sealed {}

    impl Sealed for CheckTxMode {}
    impl Sealed for ReCheckTxMode {}
    impl Sealed for DeliverTxMode {}
}
