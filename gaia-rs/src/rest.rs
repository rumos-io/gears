use axum::Router;
use bank::{BankNodeQueryRequest, BankNodeQueryResponse};
use gears::baseapp::NodeQueryHandler;
use gears::{
    baseapp::{QueryRequest, QueryResponse},
    rest::RestState,
};

pub fn get_router<
    QReq: QueryRequest + From<BankNodeQueryRequest>,
    QRes: QueryResponse + TryInto<BankNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new().nest("/cosmos/bank", bank::rest::get_router())
}
