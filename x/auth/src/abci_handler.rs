use gears::application::handlers::node::{ABCIHandler, TxError};
use gears::baseapp::errors::QueryError;
use gears::baseapp::QueryRequest;
use gears::context::init::InitContext;
use gears::context::query::QueryContext;
use gears::context::tx::TxContext;
use gears::core::Protobuf as _;
use gears::derive::Query;
use gears::params::ParamsSubspaceKey;
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::tx::raw::TxWithRaw;
use gears::types::tx::NullTxMsg;
use gears::x::module::Module;
use serde::Serialize;

use crate::query::{
    QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest, QueryAccountsResponse,
    QueryParamsRequest, QueryParamsResponse,
};
use crate::{GenesisState, Keeper};

#[derive(Clone, Debug, Query)]
#[query(request)]
pub enum AuthNodeQueryRequest {
    Account(QueryAccountRequest),
    Accounts(QueryAccountsRequest),
    Params(QueryParamsRequest),
}

impl QueryRequest for AuthNodeQueryRequest {
    fn height(&self) -> u32 {
        todo!()
    }
}

#[derive(Clone, Serialize, Query)]
#[query(response)]
#[serde(untagged)]
pub enum AuthNodeQueryResponse {
    Account(QueryAccountResponse),
    Accounts(QueryAccountsResponse),
    Params(QueryParamsResponse),
}

#[derive(Debug, Clone)]
pub struct AuthABCIHandler<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module> {
    keeper: Keeper<SK, PSK, M>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module> ABCIHandler for AuthABCIHandler<SK, PSK, M> {
    type Message = NullTxMsg;

    type Genesis = GenesisState;

    type StoreKey = SK;

    type QReq = AuthNodeQueryRequest;

    type QRes = AuthNodeQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: Self::QReq,
    ) -> Self::QRes {
        match query {
            AuthNodeQueryRequest::Account(req) => {
                let res = self.keeper.query_account(ctx, req);
                AuthNodeQueryResponse::Account(res)
            }
            AuthNodeQueryRequest::Accounts(req) => {
                let res = self.keeper.query_accounts(ctx, req);
                AuthNodeQueryResponse::Accounts(res)
            }
            AuthNodeQueryRequest::Params(req) => {
                let res = self.keeper.query_params(ctx, req);
                AuthNodeQueryResponse::Params(res)
            }
        }
    }

    fn run_ante_checks<DB: Database>(
        &self,
        _: &mut TxContext<'_, DB, Self::StoreKey>,
        _: &TxWithRaw<Self::Message>,
        _: bool,
    ) -> Result<(), TxError> {
        Ok(())
    }

    fn msg<DB: Database>(
        &self,
        _: &mut TxContext<'_, DB, Self::StoreKey>,
        _: &Self::Message,
    ) -> Result<(), TxError> {
        unreachable!("auth doesn't contain any tx")
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        self.keeper.init_genesis(ctx, genesis);

        Vec::new()
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: RequestQuery,
    ) -> Result<Vec<u8>, QueryError> {
        match query.path.as_str() {
            "/cosmos.auth.v1beta1.Query/Account" => {
                let req = QueryAccountRequest::decode(query.data)?;

                Ok(self.keeper.query_account(ctx, req).encode_vec())
            }
            "/cosmos.auth.v1beta1.Query/Accounts" => {
                let req = QueryAccountsRequest::decode(query.data)?;

                Ok(self.keeper.query_accounts(ctx, req).encode_vec())
            }
            "/cosmos.auth.v1beta1.Query/Params" => {
                let req = QueryParamsRequest::decode(query.data)?;

                Ok(self.keeper.query_params(ctx, req).encode_vec())
            }
            _ => Err(QueryError::PathNotFound),
        }
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module> AuthABCIHandler<SK, PSK, M> {
    pub fn new(keeper: Keeper<SK, PSK, M>) -> Self {
        AuthABCIHandler { keeper }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }
}
