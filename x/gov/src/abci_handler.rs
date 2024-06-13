use bytes::Bytes;
use gears::{
    application::handlers::node::ABCIHandler,
    context::{init::InitContext, query::QueryContext, tx::TxContext},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::request::query::RequestQuery,
    types::tx::raw::TxWithRaw,
    x::{keepers::bank::BankKeeper, module::Module},
};

use crate::{
    genesis::GovGenesisState,
    keeper::GovKeeper,
    msg::GovMsg,
    query::{GovQueryRequest, GovQueryResponse},
};

#[derive(Debug, Clone)]
pub struct GovAbciHandler<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> {
    _keeper: GovKeeper<SK, PSK, M, BK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>>
    GovAbciHandler<SK, PSK, M, BK>
{
    pub fn new(_keeper: GovKeeper<SK, PSK, M, BK>) -> Self {
        Self { _keeper }
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> ABCIHandler
    for GovAbciHandler<SK, PSK, M, BK>
{
    type Message = GovMsg;

    type Genesis = GovGenesisState;

    type StoreKey = SK;

    type QReq = GovQueryRequest;

    type QRes = GovQueryResponse;

    fn typed_query<DB: Database + Send + Sync>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        todo!()
    }

    fn run_ante_checks<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        _tx: &TxWithRaw<Self::Message>,
    ) -> Result<(), gears::error::AppError> {
        todo!()
    }

    fn tx<DB: gears::store::database::Database + Sync + Send>(
        &self,
        _ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        _msg: &Self::Message,
    ) -> Result<(), gears::error::AppError> {
        todo!()
    }

    fn init_genesis<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        _genesis: Self::Genesis,
    ) {
        todo!()
    }

    fn query<DB: gears::store::database::Database + Send + Sync>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: RequestQuery,
    ) -> Result<Bytes, gears::error::AppError> {
        todo!()
    }
}
