use std::marker::PhantomData;

use gears::{
    application::handlers::node::{ABCIHandler, ModuleInfo},
    baseapp::genesis::NullGenesis,
    params::ParamsSubspaceKey,
    store::StoreKey,
    types::tx::NullTxMsg,
    x::module::Module,
};

use crate::types::query::{UpgradeQueryRequest, UpgradeQueryResponse};

#[derive(Debug, Clone)]
pub struct UpgradeAbciHandler<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, MI> {
    _marker: PhantomData<(MI, SK, PSK, M)>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, MI: ModuleInfo> ABCIHandler
    for UpgradeAbciHandler<SK, PSK, M, MI>
{
    type Message = NullTxMsg;

    type Genesis = NullGenesis;

    type StoreKey = SK;

    type QReq = UpgradeQueryRequest;

    type QRes = UpgradeQueryResponse;

    fn typed_query<DB: gears::store::database::Database>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        todo!()
    }

    fn run_ante_checks<DB: gears::store::database::Database>(
        &self,
        _: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _: &gears::types::tx::raw::TxWithRaw<Self::Message>,
        _: bool,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        Ok(())
    }

    fn msg<DB: gears::store::database::Database>(
        &self,
        _: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _: &Self::Message,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        Ok(())
    }

    fn init_genesis<DB: gears::store::database::Database>(
        &self,
        _: &mut gears::context::init::InitContext<'_, DB, Self::StoreKey>,
        _: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        Vec::new()
    }

    fn query<DB: gears::store::database::Database + Send + Sync>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: gears::tendermint::types::request::query::RequestQuery,
    ) -> Result<Vec<u8>, gears::baseapp::errors::QueryError> {
        todo!()
    }

    fn begin_block<'a, DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::request::RequestBeginBlock,
    ) {
    }

    fn end_block<'a, DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::request::RequestEndBlock,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        Vec::new()
    }
}
