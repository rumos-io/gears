use axum::Router;
use bank::{BankNodeQueryRequest, BankNodeQueryResponse};
use gears::application::handlers::node::ABCIHandler;
use gears::store::StoreKey;
use gears::types::tx::TxMessage;
use gears::{
    application::ApplicationInfo,
    baseapp::{genesis::Genesis, QueryRequest, QueryResponse},
    params::ParamsSubspaceKey,
    rest::RestState,
};

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest + From<BankNodeQueryRequest>,
    QRes: QueryResponse + TryInto<BankNodeQueryResponse>,
>() -> Router<RestState<SK, PSK, M, H, G, AI, QReq, QRes>> {
    Router::new().nest("/cosmos/bank", bank::rest::get_router())
}
