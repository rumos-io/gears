use auth::{AuthNodeQueryRequest, AuthNodeQueryResponse};
use axum::Router;
use bank::{BankNodeQueryRequest, BankNodeQueryResponse};
use distribution::{DistributionNodeQueryRequest, DistributionNodeQueryResponse};
use gears::baseapp::NodeQueryHandler;
use gears::{
    baseapp::{QueryRequest, QueryResponse},
    rest::RestState,
};
use slashing::{SlashingNodeQueryRequest, SlashingNodeQueryResponse};
use staking::{StakingNodeQueryRequest, StakingNodeQueryResponse};

pub fn get_router<
    QReq: QueryRequest
        + From<AuthNodeQueryRequest>
        + From<BankNodeQueryRequest>
        + From<StakingNodeQueryRequest>
        + From<SlashingNodeQueryRequest>
        + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse
        + TryInto<AuthNodeQueryResponse>
        + TryInto<BankNodeQueryResponse>
        + TryInto<StakingNodeQueryResponse>
        + TryInto<SlashingNodeQueryResponse>
        + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .nest("/cosmos/bank", bank::rest::get_router())
        .nest("/cosmos/auth", auth::rest::get_router())
        .nest("/cosmos/staking", staking::rest::get_router())
        .nest("/cosmos/slashing", slashing::rest::get_router())
        .nest("/cosmos/distribution", distribution::rest::get_router())
}
