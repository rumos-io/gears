use kv_store::types::query::QueryMultiStore;
use serde::Serialize;

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    error::POISONED_LOCK,
    params::ParamsSubspaceKey,
    types::context::query::QueryContext,
};

use super::{errors::QueryError, BaseApp};

pub trait QueryRequest: Clone + Send + Sync + 'static {
    fn height(&self) -> u32;
}

pub trait QueryResponse: Clone + Send + Sync + 'static + Serialize {}

pub trait NodeQueryHandler<QReq, QRes>: Clone + Send + Sync + 'static {
    fn typed_query<Q: Into<QReq>>(&self, request: Q) -> Result<QRes, QueryError>;
}

impl<PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo> NodeQueryHandler<H::QReq, H::QRes>
    for BaseApp<PSK, H, AI>
{
    fn typed_query<Q: Into<H::QReq>>(&self, request: Q) -> Result<H::QRes, QueryError> {
        let request = request.into();
        let version = request.height();

        let query_store =
            QueryMultiStore::new(&*self.multi_store.read().expect(POISONED_LOCK), version)?;

        let ctx = QueryContext::new(query_store, version)?;

        Ok(self.abci_handler.typed_query(&ctx, request))
    }
}
