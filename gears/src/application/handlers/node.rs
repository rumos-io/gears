use crate::{
    error::AppError,
    signing::renderer::value_renderer::ValueRenderer,
    types::{
        context::{init::InitContext, query::QueryContext, tx::TxContext, TransactionalContext},
        tx::{raw::TxWithRaw, TxMessage},
    },
};
use database::Database;
use serde::de::DeserializeOwned;
use store_crate::StoreKey;
use tendermint::types::{
    proto::validator::ValidatorUpdate,
    request::{begin_block::RequestBeginBlock, end_block::RequestEndBlock, query::RequestQuery},
};

pub trait AnteHandlerTrait<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn run<DB: Database, M: TxMessage + ValueRenderer>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError>;
}

pub trait ABCIHandler<
    M: TxMessage,
    SK: StoreKey,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
    QReq,
    QRes,
>: Clone + Send + Sync + 'static
{
    fn typed_query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QReq,
    ) -> Result<QRes, AppError>;

    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError>;

    fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &M,
    ) -> Result<(), AppError>;

    #[allow(unused_variables)]
    fn begin_block<'a, DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        request: RequestBeginBlock,
    ) {
    }

    #[allow(unused_variables)]
    fn end_block<'a, DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        vec![]
    }

    fn init_genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: G);

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError>;
}
