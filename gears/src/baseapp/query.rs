use database::Database;
use kv::query::QueryMultiStore;
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

        let query_store = QueryMultiStore::new(
            &self.state.read().expect(POISONED_LOCK).multi_store,
            version,
        )?;

        let ctx = QueryContext::new(query_store, version)?;

        Ok(self.abci_handler.typed_query(&ctx, request))
    }
}
