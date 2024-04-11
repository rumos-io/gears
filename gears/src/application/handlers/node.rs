use crate::{
    error::AppError,
    types::{
        context::{
            init_context::InitContext, query_context::QueryContext, tx_context::TxContext,
            TransactionalContext,
        },
        tx::{raw::TxWithRaw, TxMessage},
    },
    x::signing::renderer::value_renderer::ValueRenderer,
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
    fn run<
        DB: Database,
        M: TxMessage + ValueRenderer,
        CTX: TransactionalContext<PrefixDB<DB>, SK>,
    >(
        &self,
        ctx: &mut CTX,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError>;
}

pub trait ABCIHandler<
    M: TxMessage,
    SK: StoreKey,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>: Clone + Send + Sync + 'static
{
    fn run_ante_checks<DB: Database, CTX: TransactionalContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &mut CTX,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError>;

    fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &M,
    ) -> Result<(), AppError>;

    #[allow(unused_variables)]
    fn begin_block<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        request: RequestBeginBlock,
    ) {
    }

    #[allow(unused_variables)]
    fn end_block<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
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
