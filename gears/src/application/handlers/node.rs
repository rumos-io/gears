use crate::{
    error::AppError,
    signing::renderer::value_renderer::ValueRenderer,
    types::{
        context::{
            init_context::InitContext, query_context::QueryContext, tx::TxContext,
            TransactionalContext,
        },
        tx::{raw::TxWithRaw, TxMessage},
    },
};
use serde::de::DeserializeOwned;
use store_crate::{
    database::{Database, PrefixDB},
    StoreKey,
};
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
>: Clone + Send + Sync + 'static
{
    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError>;

    fn tx<DB: Database + Sync + Send, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        msg: &M,
    ) -> Result<(), AppError>;

    #[allow(unused_variables)]
    fn begin_block<DB: Database, CTX: TransactionalContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &mut CTX,
        request: RequestBeginBlock,
    ) {
    }

    #[allow(unused_variables)]
    fn end_block<DB: Database, CTX: TransactionalContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &mut CTX,
        request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        vec![]
    }

    fn init_genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: G);

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError>;
}
