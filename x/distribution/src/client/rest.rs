use crate::{
    DistributionNodeQueryRequest, DistributionNodeQueryResponse, DistributionParams,
    QueryCommunityPoolRequest, QueryCommunityPoolResponse, QueryDelegatorParams,
    QueryParamsRequest, QueryParamsResponse,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
    types::address::AccAddress,
};

pub async fn delegation_delegator_rewards<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(delegator_address): Path<AccAddress>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::DelegatorTotalRewards(QueryDelegatorParams {
        delegator_address,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn community_pool<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::CommunityPool(QueryCommunityPoolRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn const_community_pool() -> Result<Json<QueryCommunityPoolResponse>, HTTPError> {
    let res = QueryCommunityPoolResponse {
        pool: gears::types::base::coins::DecimalCoins::new(vec![
            gears::types::base::coin::DecimalCoin {
                denom: "uatom".try_into().expect("hardcoded value cannot fail"),
                amount: gears::types::decimal256::Decimal256::from_atomics(10_000_000_000u64, 1)
                    .unwrap(),
            },
        ])
        .ok(),
    };
    Ok(Json(res))
}

pub async fn params<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::Params(QueryParamsRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn const_params() -> Result<Json<QueryParamsResponse>, HTTPError> {
    let res = QueryParamsResponse {
        params: DistributionParams::default(),
    };
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        // TODO: remove const handler and route after integration and update route
        .route("/v1beta1/community_pool/current", get(community_pool))
        .route("/v1beta1/community_pool", get(const_community_pool))
        // TODO: check path
        .route(
            "/v1beta1/delegators/:delegator_address/rewards",
            get(delegation_delegator_rewards),
        )
        // TODO: remove const handler and route after integration and update route
        .route("/v1beta1/params/current", get(params))
        .route("/v1beta1/params", get(const_params))
}
