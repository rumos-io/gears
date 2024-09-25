use database::Database;
use kv_store::query::QueryMultiStore;
use serde::Serialize;

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    context::query::QueryContext,
    error::POISONED_LOCK,
    params::ParamsSubspaceKey,
};

use super::{errors::QueryError, BaseApp};

/// Return url which could be used to query this... query
pub trait Query {
    fn query_url(&self) -> &'static str;
    fn into_bytes(self) -> Vec<u8>;
}

pub trait QueryRequest: Clone + Send + Sync + 'static {
    fn height(&self) -> u32;
}

pub trait QueryResponse: Clone + Send + Sync + 'static + Serialize {
    fn into_bytes(self) -> Vec<u8>;
}

pub trait NodeQueryHandler<QReq, QRes>: Clone + Send + Sync + 'static {
    fn typed_query<Q: Into<QReq>>(&self, request: Q) -> Result<QRes, QueryError>;
}

impl<DB: Database, PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo>
    NodeQueryHandler<H::QReq, H::QRes> for BaseApp<DB, PSK, H, AI>
{
    fn typed_query<Q: Into<H::QReq>>(&self, request: Q) -> Result<H::QRes, QueryError> {
        let request = request.into();
        let version = request.height();

        let store = self.multi_store.read().expect(POISONED_LOCK);
        let ctx = QueryContext::new(QueryMultiStore::new(&*store, version)?, version)?;
        Ok(self.abci_handler.typed_query(&ctx, request))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NullQueryRequest {}

impl Query for NullQueryRequest {
    fn query_url(&self) -> &'static str {
        unreachable!()
    }

    fn into_bytes(self) -> Vec<u8> {
        unreachable!()
    }
}

impl QueryRequest for NullQueryRequest {
    fn height(&self) -> u32 {
        unreachable!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum NullQueryResponse {}

impl QueryResponse for NullQueryResponse {
    fn into_bytes(self) -> Vec<u8> {
        unreachable!()
    }
}
