use serde::Serialize;
use store_crate::{types::query::QueryMultiStore, StoreKey};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    error::{AppError, POISONED_LOCK},
    params::ParamsSubspaceKey,
    types::{context::query::QueryContext, tx::TxMessage},
};

use super::{genesis::Genesis, BaseApp};

pub trait QueryRequest: Clone + Send + Sync + 'static {
    fn height(&self) -> u32;
}

pub trait QueryResponse: Clone + Send + Sync + 'static + Serialize {}

pub trait NodeQueryHandler<QReq, QRes>: Clone + Send + Sync + 'static {
    fn typed_query<Q: Into<QReq>>(&self, request: Q) -> Result<QRes, AppError>;
}

impl<
        M: TxMessage,
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        H: ABCIHandler<M, SK, G, QReq, QRes>,
        G: Genesis,
        AI: ApplicationInfo,
        QReq: QueryRequest,
        QRes: QueryResponse,
    > NodeQueryHandler<QReq, QRes> for BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>
{
    fn typed_query<Q: Into<QReq>>(&self, request: Q) -> Result<QRes, AppError> {
        // TODO: use a specific query error type that can be converted to HTTP error
        let request = request.into();
        let version = request.height();

        let query_store =
            QueryMultiStore::new(&*self.multi_store.read().expect(POISONED_LOCK), version)?;

        let ctx = QueryContext::new(query_store, version)?;

        self.abci_handler.typed_query(&ctx, request)
    }
}
