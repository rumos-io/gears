use std::marker::PhantomData;

use gears::{
    application::handlers::node::ABCIHandler,
    baseapp::{NullQueryRequest, NullQueryResponse},
    store::StoreKey,
    types::tx::NullTxMsg,
};

use crate::genesis::GenutilGenesis;

#[derive(Debug, Clone)]
pub struct GenutilAbciHandler<SK> {
    _sk_marker: PhantomData<SK>,
}

impl<SK: StoreKey> ABCIHandler for GenutilAbciHandler<SK> {
    type Message = NullTxMsg;

    type Genesis = GenutilGenesis;

    type StoreKey = SK;

    type QReq = NullQueryRequest;

    type QRes = NullQueryResponse;

    fn typed_query<DB: gears::store::database::Database>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        unreachable!()
    }

    fn run_ante_checks<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _tx: &gears::types::tx::raw::TxWithRaw<Self::Message>,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        Ok(())
    }

    fn msg<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _msg: &Self::Message,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        unreachable!()
    }

    fn init_genesis<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::init::InitContext<'_, DB, Self::StoreKey>,
        _genesis: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        todo!()
    }

    fn query<DB: gears::store::database::Database + Send + Sync>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: gears::tendermint::types::request::query::RequestQuery,
    ) -> Result<Vec<u8>, gears::baseapp::errors::QueryError> {
        unreachable!()
    }
}
