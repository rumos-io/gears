use crate::{
    baseapp::genesis::Genesis,
    error::AppError,
    signing::renderer::value_renderer::ValueRenderer,
    types::{
        context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
        tx::{raw::TxWithRaw, TxMessage},
    },
};
use database::Database;
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

pub trait ABCIHandler: Clone + Send + Sync + 'static {
    type Message: TxMessage;
    type Genesis: Genesis;
    type StoreKey: StoreKey;

    fn typed_query<DB: Database + Send + Sync, QReq, QRes>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: QReq,
    ) -> QRes;

    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        tx: &TxWithRaw<Self::Message>,
    ) -> Result<(), AppError>;

    fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        msg: &Self::Message,
    ) -> Result<(), AppError>;

    #[allow(unused_variables)]
    fn begin_block<'a, DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, Self::StoreKey>,
        request: RequestBeginBlock,
    ) {
    }

    #[allow(unused_variables)]
    fn end_block<'a, DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, Self::StoreKey>,
        request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        Vec::new()
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    );

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError>;
}
