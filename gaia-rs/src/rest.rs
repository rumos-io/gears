use auth::{AuthNodeQueryRequest, AuthNodeQueryResponse};
use axum::Router;
use bank::{BankNodeQueryRequest, BankNodeQueryResponse};
use gears::baseapp::NodeQueryHandler;
use gears::{
    baseapp::{QueryRequest, QueryResponse},
    rest::RestState,
};
use staking::{StakingNodeQueryRequest, StakingNodeQueryResponse};

pub fn get_router<
    QReq: QueryRequest
        + From<AuthNodeQueryRequest>
        + From<BankNodeQueryRequest>
        + From<StakingNodeQueryRequest>,
    QRes: QueryResponse
        + TryInto<AuthNodeQueryResponse>
        + TryInto<BankNodeQueryResponse>
        + TryInto<StakingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .nest("/cosmos/bank", bank::rest::get_router())
        .nest("/cosmos/auth", auth::rest::get_router())
        .nest("/cosmos/staking", staking::rest::get_router())
}
